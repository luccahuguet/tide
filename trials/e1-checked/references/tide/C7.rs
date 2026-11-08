pub struct BoundedStack<T> {
    data: Vec<T>,
    capacity: usize,
}
impl<T: Pure + Sync> BoundedStack<T> {
    #[requires(capacity >= 1)]
    pub fn new(capacity: usize) -> Self {
        Self {
            data: Vec::new(),
            capacity,
        }
    }
    #[requires(!self.is_full())]
    #[ensures(self.len() == old(self.len()) + 1)]
    pub fn push(&mut self, x: T) {
        self.data.push(x);
    }
    #[ensures(self.len() == old(self.len()).saturating_sub(1))]
    pub fn pop(&mut self) -> Option<T> {
        self.data.pop()
    }
    pub fn peek(&self) -> Option<&T> {
        self.data.last()
    }
    pub fn len(&self) -> usize {
        self.data.len()
    }
    pub fn capacity(&self) -> usize {
        self.capacity
    }
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
    pub fn is_full(&self) -> bool {
        self.len() == self.capacity
    }
    #[requires(n <= self.len())]
    pub fn drain_top(&mut self, n: usize) -> Vec<T> {
        (0..n).map(|_| self.pop().unwrap()).collect()
    }
}
