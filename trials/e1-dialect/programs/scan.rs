// Task 3: scan -- src/lib.rs
//
// Byte scanning (Boyer-Moore-Horspool) and borrowed line iteration.

/// A search pattern with its Horspool skip table, owning a copy of the pattern.
pub struct Needle {
    pat: Vec<u8>,
    skip: [usize; 256],
}

impl Needle {
    pub fn new(pattern: &str) -> Needle {
        let pat = pattern.as_bytes().to_vec();
        let mut skip = [pat.len(); 256];
        if let Some((_, init)) = pat.split_last() {
            for (i, &b) in init.iter().enumerate() {
                skip[usize::from(b)] = pat.len() - 1 - i;
            }
        }
        Needle { pat, skip }
    }
}

/// Start offsets of non-overlapping matches. Owns its copy of the needle's
/// data, so it borrows only the haystack.
pub struct Positions<'h> {
    hay: &'h [u8],
    pat: Vec<u8>,
    skip: [usize; 256],
    at: usize,
}

/// A lazy iterator over the start offsets of non-overlapping matches of
/// `needle` in `haystack`. An empty needle matches nowhere.
pub fn positions<'h>(haystack: &'h [u8], needle: &Needle) -> Positions<'h> {
    Positions { hay: haystack, pat: needle.pat.clone(), skip: needle.skip, at: 0 }
}

impl Iterator for Positions<'_> {
    type Item = usize;

    fn next(&mut self) -> Option<usize> {
        let m = self.pat.len();
        if m == 0 {
            return None;
        }
        while self.at + m <= self.hay.len() {
            let start = self.at;
            if self.hay[start..start + m] == self.pat[..] {
                self.at = start + m; // non-overlapping: resume after the match
                return Some(start);
            }
            self.at = start + self.skip[usize::from(self.hay[start + m - 1])];
        }
        None
    }
}

/// The lines of a text, without terminators; `\n` and `\r\n` both end a line.
#[derive(Debug, Clone)]
pub struct Lines<'a> {
    rest: &'a str,
}

impl<'a> Lines<'a> {
    pub fn new(text: &'a str) -> Self {
        Lines { rest: text }
    }

    /// The remaining lines numbered from 1, without consuming `self`.
    pub fn numbered(&self) -> impl Iterator<Item = (usize, &'a str)> + use<'a> {
        self.clone().enumerate().map(|(i, line)| (i + 1, line))
    }

    /// The remaining lines that contain `word`, without consuming `self`.
    pub fn containing<'w>(&self, word: &'w str) -> impl Iterator<Item = &'a str> + use<'a, 'w> {
        self.clone().filter(move |line| line.contains(word))
    }
}

impl<'a> Iterator for Lines<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<&'a str> {
        if self.rest.is_empty() {
            return None;
        }
        let line = match self.rest.split_once('\n') {
            Some((line, rest)) => {
                self.rest = rest;
                line.strip_suffix('\r').unwrap_or(line)
            }
            None => {
                let line = self.rest;
                self.rest = "";
                line
            }
        };
        Some(line)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn positions_on_overlapping_candidates() {
        let n = Needle::new("aaaa");
        let got: Vec<usize> = positions(b"aaaaaaaaa", &n).collect();
        assert_eq!(got, vec![0, 4]);
    }

    #[test]
    fn positions_with_skips() {
        let n = Needle::new("abc");
        assert_eq!(positions(b"xxabcabcab", &n).collect::<Vec<_>>(), vec![2, 5]);
        assert_eq!(positions(b"ab", &n).count(), 0);
        assert_eq!(positions(b"abc", &Needle::new("")).count(), 0);
    }

    #[test]
    fn positions_outlive_the_needle() {
        let hay = String::from("one two one");
        let it;
        {
            let needle = Needle::new("one");
            it = positions(hay.as_bytes(), &needle);
        }
        assert_eq!(it.collect::<Vec<_>>(), vec![0, 8]);
    }

    #[test]
    fn numbered_and_containing_with_crlf() {
        let lines = Lines::new("first line\r\nsecond\r\nthird\r\n");
        let numbered: Vec<(usize, &str)> = lines.numbered().collect();
        assert_eq!(numbered, vec![(1, "first line"), (2, "second"), (3, "third")]);
        let hits: Vec<&str> = lines.containing("ir").collect();
        assert_eq!(hits, vec!["first line", "third"]);
        assert_eq!(lines.count(), 3); // still ours: neither call consumed it
    }

    #[test]
    fn mixed_terminators_and_unterminated_last_line() {
        let got: Vec<&str> = Lines::new("a\nb\r\n\nc").collect();
        assert_eq!(got, vec!["a", "b", "", "c"]);
        assert_eq!(Lines::new("").count(), 0);
    }

    #[test]
    fn containing_with_a_local_string_word() {
        let text = String::from("alpha\nbeta\ngamma alpha");
        let lines = Lines::new(&text);
        let hits: Vec<&str>;
        {
            let word = String::from("alpha");
            hits = lines.containing(&word).collect();
        }
        assert_eq!(hits, vec!["alpha", "gamma alpha"]);
    }
}
