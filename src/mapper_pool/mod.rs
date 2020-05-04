#![allow(dead_code)]

use std::iter::FromIterator;
use std::sync::Mutex;

mod pool;
use pool::Pool;

mod mapper;
pub use mapper::Mapper;

use scoped_pool::Pool as ThreadPool;

pub struct MapperPool<T> {
    pub(self) pool: Mutex<Pool<Mapper<T>>>,
}

impl<T> MapperPool<T>
where
    T: Send,
{
    pub fn builder() -> MapperPoolBuilder {
        Default::default()
    }

    pub fn map<I, It, Item, O, F>(&self, func: F, iter: I) -> O
    where
        Item: Send,
        I: IntoIterator<IntoIter = It, Item = Item>,
        It: Iterator<Item = Item> + ExactSizeIterator,
        O: FromIterator<T>,
        F: Fn(Item) -> T + Send + Sync,
    {
        let mut mapper = self.pool.lock().unwrap().get();
        let res = mapper.map(func, iter);
        self.pool.lock().unwrap().put(mapper);
        res
    }
}

pub struct MapperPoolBuilder {
    pub pool_size: usize,
    pub storage_size: usize,
    pub workers_count: usize,
}

impl MapperPoolBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn pool_size(&mut self, val: usize) -> &mut Self {
        self.pool_size = val;
        self
    }

    pub fn storage_size(&mut self, val: usize) -> &mut Self {
        self.storage_size = val;
        self
    }

    pub fn workers_count(&mut self, val: usize) -> &mut Self {
        self.workers_count = val;
        self
    }

    pub fn build<T>(&self) -> MapperPool<T>
    where
        T: Send,
    {
        let threadpool = ThreadPool::new(self.workers_count);
        let storage_size = self.storage_size;
        MapperPool {
            pool: Mutex::new(Pool::new(self.pool_size, move || {
                Mapper::new(threadpool.clone(), storage_size)
            })),
        }
    }
}

impl Default for MapperPoolBuilder {
    fn default() -> Self {
        Self {
            pool_size: 0,
            storage_size: 0,
            workers_count: num_cpus::get(),
        }
    }
}
