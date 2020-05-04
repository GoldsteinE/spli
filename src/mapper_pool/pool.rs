pub struct Pool<T> {
    pool: Vec<T>,
    factory: Box<dyn Fn() -> T + 'static>,
}

impl<T> Pool<T> {
    pub fn new<F>(pool_size: usize, factory: F) -> Self
    where
        F: Fn() -> T + 'static,
    {
        let mut result = Self {
            pool: Vec::new(),
            factory: Box::new(factory),
        };
        result.put_n(pool_size);
        result
    }

    pub fn get(&mut self) -> T {
        self.pool.pop().unwrap_or_else(&self.factory)
    }

    pub fn put(&mut self, new: T) {
        self.pool.push(new)
    }

    pub fn put_n(&mut self, count: usize) {
        self.pool.reserve_exact(count);
        for _ in 0..count {
            self.pool.push((self.factory)())
        }
    }
}
