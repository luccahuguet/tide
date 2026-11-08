pub struct BoundedStack<T> {
    data: Vec<T>,
    capacity: usize,
}
impl<T> BoundedStack<T> {
    pub fn new(capacity: usize) -> Self {
        assert!(capacity >= 1);
        Self {
            data: Vec::new(),
            capacity,
        }
    }
    pub fn push(&mut self, x: T) {
        assert!(!self.is_full());
        self.data.push(x);
    }
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
    pub fn drain_top(&mut self, n: usize) -> Vec<T> {
        assert!(n <= self.len());
        (0..n).map(|_| self.pop().unwrap()).collect()
    }
}
