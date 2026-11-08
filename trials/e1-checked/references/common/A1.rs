use std::collections::HashMap;

fn key(word: &str) -> Vec<u8> {
    let mut bytes = word.to_ascii_lowercase().into_bytes();
    bytes.sort();
    bytes
}

pub struct AnagramIndex<'a> {
    classes: Vec<Vec<&'a str>>,
}
impl<'a> AnagramIndex<'a> {
    pub fn new(text: &'a str) -> Self {
        let mut classes: Vec<Vec<&str>> = Vec::new();
        let mut positions = HashMap::new();
        for word in text
            .split(|c: char| !c.is_ascii_alphabetic())
            .filter(|s| !s.is_empty())
        {
            let next = classes.len();
            let i = *positions.entry(key(word)).or_insert_with(|| {
                classes.push(Vec::new());
                next
            });
            if !classes[i].contains(&word) {
                classes[i].push(word);
            }
        }
        Self { classes }
    }
    pub fn groups(&self) -> Vec<Vec<&'a str>> {
        self.classes
            .iter()
            .filter(|g| g.len() > 1)
            .cloned()
            .collect()
    }
    pub fn partners(&self, word: &str) -> Vec<&'a str> {
        let k = key(word);
        self.classes
            .iter()
            .flatten()
            .copied()
            .filter(|w| *w != word && key(w) == k)
            .collect()
    }
}
