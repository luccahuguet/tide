//@ rules: LIFE-4
// Rule 1 (one input origin, elided or named) and rule 2 (the receiver's origin wins).
pub fn first_word(s: &str) -> &str {
    s
}

pub fn named<'a>(x: &'a u8, _n: u8) -> &u8 {
    x
}

pub struct Table {
    keys: Vec<String>,
}

impl Table {
    pub fn get(&self, _k: &str) -> Option<&String> {
        self.keys.first()
    }
}
