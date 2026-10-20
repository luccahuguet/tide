// Task 1: canon -- src/lib.rs
//
// A dictionary of canonical spellings. Every spelling it hands out is a slice
// of the build text, so results never borrow the `Dict` itself.

use std::collections::HashMap;

/// Canonical spellings, keyed by ASCII-lowercased word.
pub struct Dict<'t> {
    first: HashMap<String, &'t str>,
}

impl<'t> Dict<'t> {
    /// Records, for each word of `text` compared case-insensitively (ASCII),
    /// the first spelling seen.
    pub fn build(text: &'t str) -> Self {
        let mut first = HashMap::new();
        for word in text.split_whitespace() {
            first.entry(word.to_ascii_lowercase()).or_insert(word);
        }
        Dict { first }
    }

    /// The canonical spelling of `word`, as a slice of the build text.
    pub fn get(&self, word: &str) -> Option<&'t str> {
        self.first.get(&word.to_ascii_lowercase()).copied()
    }

    /// The canonical spelling if there is one, else `word` itself. The result
    /// borrows from the build text or from `word`, never from `self`.
    pub fn canonical_or<'r>(&self, word: &'r str) -> &'r str
    where
        't: 'r,
    {
        self.get(word).unwrap_or(word)
    }

    /// The words of `line`, each replaced by its canonical spelling where one
    /// exists.
    pub fn rewrite<'r>(&self, line: &'r str) -> Vec<&'r str>
    where
        't: 'r,
    {
        line.split_whitespace().map(|w| self.canonical_or(w)).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_and_canonical_or_on_mixed_case() {
        let text = "Hello World hello WORLD Rust";
        let d = Dict::build(text);
        assert_eq!(d.get("HELLO"), Some("Hello"));
        assert_eq!(d.get("world"), Some("World"));
        assert_eq!(d.get("rUsT"), Some("Rust"));
        assert_eq!(d.get("missing"), None);
        assert_eq!(d.canonical_or("wOrLd"), "World");
        assert_eq!(d.canonical_or("Other"), "Other");
    }

    #[test]
    fn get_result_outlives_dict() {
        let text = String::from("Alpha beta");
        let got;
        {
            let d = Dict::build(&text);
            got = d.get("ALPHA");
        }
        assert_eq!(got, Some("Alpha"));
    }

    #[test]
    fn canonical_or_result_outlives_dict() {
        let text = String::from("Alpha beta");
        {
            let other = String::from("ALPHA gamma");
            let d = Dict::build(&text);
            let mut words = other.split_whitespace();
            let known = d.canonical_or(words.next().unwrap());
            let unknown = d.canonical_or(words.next().unwrap());
            drop(d);
            // `other` is still alive; the Dict is gone.
            assert_eq!(known, "Alpha");
            assert_eq!(unknown, "gamma");
        }
    }

    #[test]
    fn rewrite_result_outlives_dict() {
        let text = String::from("The Quick fox");
        {
            let line = String::from("the QUICK brown FOX");
            let d = Dict::build(&text);
            let out = d.rewrite(&line);
            drop(d);
            assert_eq!(out, vec!["The", "Quick", "brown", "fox"]);
        }
        let line = String::from("FOX jumps");
        let out;
        {
            let d = Dict::build(&text);
            out = d.rewrite(&line);
        }
        assert_eq!(out, vec!["fox", "jumps"]);
    }
}
