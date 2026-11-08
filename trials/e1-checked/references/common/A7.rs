use std::collections::BTreeMap;
#[derive(Default)]
struct Node {
    word: bool,
    children: BTreeMap<char, Node>,
}
pub struct Trie {
    root: Node,
    len: usize,
}
impl Trie {
    pub fn new() -> Self {
        Self {
            root: Node::default(),
            len: 0,
        }
    }
    pub fn insert(&mut self, w: &str) -> bool {
        let mut n = &mut self.root;
        for c in w.chars() {
            n = n.children.entry(c).or_default();
        }
        if n.word {
            false
        } else {
            n.word = true;
            self.len += 1;
            true
        }
    }
    fn node(&self, w: &str) -> Option<&Node> {
        let mut n = &self.root;
        for c in w.chars() {
            n = n.children.get(&c)?;
        }
        Some(n)
    }
    pub fn contains(&self, w: &str) -> bool {
        self.node(w).is_some_and(|n| n.word)
    }
    pub fn remove(&mut self, w: &str) -> bool {
        fn remove(n: &mut Node, chars: &mut std::str::Chars<'_>) -> bool {
            if let Some(c) = chars.next() {
                if let Some(child) = n.children.get_mut(&c) {
                    let removed = remove(child, chars);
                    if !child.word && child.children.is_empty() {
                        n.children.remove(&c);
                    }
                    removed
                } else {
                    false
                }
            } else {
                std::mem::replace(&mut n.word, false)
            }
        }
        let removed = remove(&mut self.root, &mut w.chars());
        if removed {
            self.len -= 1;
        }
        removed
    }
    pub fn len(&self) -> usize {
        self.len
    }
    pub fn with_prefix(&self, p: &str) -> Vec<String> {
        fn collect(n: &Node, prefix: &mut String, out: &mut Vec<String>) {
            if n.word {
                out.push(prefix.clone());
            }
            for (&c, child) in &n.children {
                prefix.push(c);
                collect(child, prefix, out);
                prefix.pop();
            }
        }
        let mut out = Vec::new();
        if let Some(n) = self.node(p) {
            collect(n, &mut p.to_string(), &mut out);
        }
        out
    }
    pub fn longest_common_prefix(&self) -> String {
        let mut s = String::new();
        let mut n = &self.root;
        while !n.word && n.children.len() == 1 {
            let (&c, child) = n.children.first_key_value().unwrap();
            s.push(c);
            n = child;
        }
        s
    }
}
