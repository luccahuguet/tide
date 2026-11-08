pub struct Doc<'a> {
    text: &'a str,
}
fn words(sentence: &str) -> Vec<&str> {
    sentence
        .split_whitespace()
        .map(|w| w.trim_matches(|c: char| c.is_ascii_punctuation()))
        .filter(|w| !w.is_empty())
        .collect()
}
impl<'a> Doc<'a> {
    pub fn new(text: &'a str) -> Self {
        Self { text }
    }
    pub fn sentences(&self) -> impl Iterator<Item = &'a str> + use<'a> {
        let mut rest = self.text;
        std::iter::from_fn(move || {
            loop {
                if rest.is_empty() {
                    return None;
                }
                let end = rest
                    .char_indices()
                    .find_map(|(i, c)| {
                        let end = i + c.len_utf8();
                        (matches!(c, '.' | '!' | '?')
                            && rest[end..].chars().next().is_none_or(char::is_whitespace))
                        .then_some(end)
                    })
                    .unwrap_or(rest.len());
                let (s, tail) = rest.split_at(end);
                rest = tail;
                let s = s.trim();
                if !s.is_empty() {
                    return Some(s);
                }
            }
        })
    }
    pub fn longest(&self) -> Option<&'a str> {
        self.sentences().fold(None, |best, s| {
            if best.is_none_or(|b: &str| s.chars().count() > b.chars().count()) {
                Some(s)
            } else {
                best
            }
        })
    }
    pub fn words_of(&self, i: usize) -> Option<Vec<&'a str>> {
        self.sentences().nth(i).map(words)
    }
    pub fn with_word<'w>(&self, word: &'w str) -> impl Iterator<Item = &'a str> + use<'a, 'w> {
        self.sentences()
            .filter(move |s| words(s).iter().any(|w| w.eq_ignore_ascii_case(word)))
    }
}
