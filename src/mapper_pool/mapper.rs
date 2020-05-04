use scoped_pool::Pool as ThreadPool;
use std::iter::FromIterator;

pub struct Mapper<T> {
    // TODO: maybe raw-pointer arrays?
    storage: Vec<Option<T>>,
    threadpool: ThreadPool,
}

impl<T> Mapper<T>
where
    T: Send,
{
    pub fn new(threadpool: ThreadPool, storage_size: usize) -> Self {
        Self {
            threadpool,
            storage: Vec::with_capacity(storage_size),
        }
    }

    pub fn map<I, It, Item, O, F>(&mut self, func: F, iter: I) -> O
    where
        I: IntoIterator<IntoIter = It, Item = Item>,
        // TODO: mb somehow lower this bound?
        It: Iterator<Item = Item> + ExactSizeIterator,
        Item: Send,
        O: FromIterator<T>,
        F: Fn(Item) -> T + Send + Sync,
    {
        let func = &func;
        let iter = iter.into_iter();
        self.storage.truncate(0);
        self.storage.resize_with(iter.len(), Default::default);

        let mut storage: &mut [Option<T>] = &mut self.storage;
        self.threadpool.scoped(move |scope| {
            for item in iter {
                let (head, tail) = storage.split_at_mut(1);
                storage = tail;
                scope.execute(move || head[0] = Some(func(item)));
            }
        });

        // TODO: mb unsafe unwrap with debug_assert!()?
        self.storage.drain(..).map(Option::unwrap).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_simple() {
        let threadpool = ThreadPool::new(4);
        let mut mapper = Mapper::new(threadpool, 16);
        assert_eq!(
            mapper.map::<_, _, _, Vec<_>, _>(|x| x.to_string(), vec![1, 2, 3, 4, 5]),
            vec![
                String::from("1"),
                String::from("2"),
                String::from("3"),
                String::from("4"),
                String::from("5"),
            ],
        );
    }
}
