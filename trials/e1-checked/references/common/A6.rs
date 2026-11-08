use std::{collections::HashMap, hash::Hash};
struct Entry<K, V> {
    key: K,
    value: V,
    prev: Option<usize>,
    next: Option<usize>,
}
pub struct Lru<K, V> {
    map: HashMap<K, usize>,
    entries: Vec<Option<Entry<K, V>>>,
    capacity: usize,
    first: Option<usize>,
    last: Option<usize>,
}
impl<K: Eq + Hash + Clone, V> Lru<K, V> {
    pub fn new(capacity: usize) -> Self {
        assert!(capacity > 0);
        Self {
            map: HashMap::new(),
            entries: Vec::new(),
            capacity,
            first: None,
            last: None,
        }
    }
    fn unlink(&mut self, i: usize) {
        let e = self.entries[i].as_ref().unwrap();
        let (p, n) = (e.prev, e.next);
        if let Some(p) = p {
            self.entries[p].as_mut().unwrap().next = n;
        } else {
            self.first = n;
        }
        if let Some(n) = n {
            self.entries[n].as_mut().unwrap().prev = p;
        } else {
            self.last = p;
        }
    }
    fn front(&mut self, i: usize) {
        let e = self.entries[i].as_mut().unwrap();
        e.prev = None;
        e.next = self.first;
        if let Some(f) = self.first {
            self.entries[f].as_mut().unwrap().prev = Some(i);
        } else {
            self.last = Some(i);
        }
        self.first = Some(i);
    }
    pub fn get(&mut self, k: &K) -> Option<&V> {
        let i = *self.map.get(k)?;
        self.unlink(i);
        self.front(i);
        Some(&self.entries[i].as_ref().unwrap().value)
    }
    pub fn peek(&self, k: &K) -> Option<&V> {
        Some(&self.entries[*self.map.get(k)?].as_ref().unwrap().value)
    }
    pub fn put(&mut self, k: K, v: V) -> Option<(K, V)> {
        if let Some(&i) = self.map.get(&k) {
            self.entries[i].as_mut().unwrap().value = v;
            self.unlink(i);
            self.front(i);
            return None;
        }
        let (i, old) = if self.map.len() == self.capacity {
            let i = self.last.unwrap();
            self.unlink(i);
            let old = self.entries[i].take().unwrap();
            self.map.remove(&old.key);
            (i, Some((old.key, old.value)))
        } else {
            self.entries.push(None);
            (self.entries.len() - 1, None)
        };
        self.map.insert(k.clone(), i);
        self.entries[i] = Some(Entry {
            key: k,
            value: v,
            prev: None,
            next: None,
        });
        self.front(i);
        old
    }
    pub fn len(&self) -> usize {
        self.map.len()
    }
    pub fn keys_recent_first(&self) -> Vec<&K> {
        let mut out = Vec::new();
        let mut i = self.first;
        while let Some(at) = i {
            let e = self.entries[at].as_ref().unwrap();
            out.push(&e.key);
            i = e.next;
        }
        out
    }
}
