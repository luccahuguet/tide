// Task 8: ring -- src/lib.rs
//
// A fixed-capacity ring buffer of i64 whose rules are contracts. Storage
// grows lazily up to the capacity, so `with_capacity(n)` costs nothing for a
// large `n` (generated contract tests draw `n` from the whole `usize` range).
// While `buf.len() < cap`, the elements occupy `buf[head..]` exactly
// (`head + len == buf.len()`), so the next slot is `buf.len()`.

pub struct Ring {
    buf: Vec<i64>,
    cap: usize,
    head: usize,
    len: usize,
}

impl Ring {
    #[requires(n >= 1)]
    #[ensures(result.len() == 0)]
    #[ensures(result.capacity() == n)]
    pub fn with_capacity(n: usize) -> Ring {
        Ring { buf: Vec::new(), cap: n, head: 0, len: 0 }
    }

    pub fn capacity(&self) -> usize {
        self.cap
    }

    #[ensures(result <= self.capacity())]
    pub fn len(&self) -> usize {
        self.len
    }

    #[ensures(result == (self.len() == self.capacity()))]
    pub fn is_full(&self) -> bool {
        self.len == self.cap
    }

    /// The caller must not push into a full ring.
    #[requires(!self.is_full())]
    #[ensures(self.len() == old(self.len()) + 1)]
    pub fn push(&mut self, x: i64) {
        let tail = (self.head + self.len) % self.cap;
        if tail == self.buf.len() {
            self.buf.push(x); // still growing: the next slot is the end of `buf`
        } else {
            self.buf[tail] = x;
        }
        self.len += 1;
    }

    /// The oldest element, or `None` exactly when the ring was empty.
    #[ensures(result.is_none() == (old(self.len()) == 0))]
    #[ensures(self.len() == old(self.len()).saturating_sub(1))]
    pub fn pop(&mut self) -> Option<i64> {
        if self.len == 0 {
            return None;
        }
        let x = self.buf[self.head];
        self.head = (self.head + 1) % self.cap;
        self.len -= 1;
        Some(x)
    }

    /// The caller must ensure that `xs` fits.
    #[requires(self.len() + xs.len() <= self.capacity())]
    #[ensures(self.len() == old(self.len()) + xs.len())]
    pub fn extend_from(&mut self, xs: &[i64]) {
        for &x in xs {
            self.push(x);
        }
    }

    /// Moves every element into `out`, oldest first.
    #[ensures(out.len() == old(out.len()) + old(self.len()))]
    #[ensures(self.len() == 0)]
    pub fn drain_into(&mut self, out: &mut Vec<i64>) {
        let n = self.len();
        let out_start = out.len();
        for moved in 0..n {
            invariant!(self.len() + moved == n);
            invariant!(out.len() == out_start + moved);
            let x = self.pop().expect("the ring holds n - moved > 0 elements");
            out.push(x);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::VecDeque;

    #[test]
    fn push_and_pop_through_a_wraparound() {
        let mut r = Ring::with_capacity(3);
        r.push(1);
        r.push(2);
        r.push(3);
        assert!(r.is_full());
        assert_eq!(r.pop(), Some(1));
        assert_eq!(r.pop(), Some(2));
        r.push(4); // wraps to slot 0
        r.push(5); // slot 1
        assert!(r.is_full());
        assert_eq!(r.len(), 3);
        assert_eq!(r.pop(), Some(3));
        assert_eq!(r.pop(), Some(4));
        assert_eq!(r.pop(), Some(5));
        assert_eq!(r.pop(), None);
        assert_eq!(r.len(), 0);
    }

    #[test]
    fn extend_and_drain() {
        let mut r = Ring::with_capacity(4);
        r.push(7);
        assert_eq!(r.pop(), Some(7));
        r.extend_from(&[1, 2, 3, 4]); // head is 1, so this wraps
        let mut out = vec![0];
        r.drain_into(&mut out);
        assert_eq!(out, vec![0, 1, 2, 3, 4]);
        assert_eq!(r.len(), 0);
        assert_eq!(r.pop(), None);
    }

    #[test]
    #[requires(cap >= 1)]
    #[requires(cap <= 8)]
    fn agrees_with_vecdeque(cap: u8, ops: Vec<Option<i64>>) {
        let cap = usize::from(cap);
        let mut ring = Ring::with_capacity(cap);
        let mut model = VecDeque::new();
        for op in ops {
            match op {
                Some(x) => {
                    if !ring.is_full() {
                        ring.push(x);
                        model.push_back(x);
                    }
                }
                None => assert_eq!(ring.pop(), model.pop_front()),
            }
            assert_eq!(ring.len(), model.len());
            assert_eq!(ring.is_full(), model.len() == cap);
        }
        let mut out = Vec::new();
        ring.drain_into(&mut out);
        assert_eq!(out, Vec::from(model));
    }

    #[test]
    #[should_panic(expected = "requires violated")]
    fn pushing_into_a_full_ring_violates_the_precondition() {
        let mut r = Ring::with_capacity(1);
        r.push(1);
        r.push(2);
    }
}
