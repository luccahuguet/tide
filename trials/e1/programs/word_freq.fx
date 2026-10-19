// src/main.fx of package `trials/word-freq`
//
// `word_freq FILE...` prints the 20 most frequent words across all files.
// Reading is sequential I/O; counting is authority-free and runs in parallel
// with `par_map`, and the per-document counts are merged by `par_reduce`,
// whose grouping depends only on the number of documents.

use std::collections::BTreeMap;
use std::fs::Fs;
use std::io;
use std::par::{Combine, Monoid};

const TOP: usize = 20;

/// Word counts. A monoid under per-word addition, with `empty()` as identity.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Counts(pub BTreeMap<String, u64>);

impl Counts {
    /// Authority-free. Words are maximal alphanumeric runs, lowercased.
    pub fn of(doc: &str) -> Counts {
        let mut map = BTreeMap::new();
        for word in doc.split(|c: char| !c.is_alphanumeric()).filter(|w| !w.is_empty()) {
            *map.entry(word.to_lowercase()).or_insert(0) += 1;
        }
        Counts(map)
    }

    /// The `n` most frequent words, ties broken alphabetically, so the output
    /// is the same on every run and core count.
    pub fn top(&self, n: usize) -> Vec<(&str, u64)> {
        let mut entries: Vec<(&str, u64)> = self.0.iter().map(|(w, &c)| (w.as_str(), c)).collect();
        entries.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(b.0)));
        entries.truncate(n);
        entries
    }
}

impl Combine for Counts {
    fn empty() -> Counts {
        Counts::default()
    }

    fn combine(&self, other: &Counts) -> Counts {
        let mut out = self.0.clone();
        for (word, n) in &other.0 {
            *out.entry(word.clone()).or_insert(0) += n;
        }
        Counts(out)
    }
}

/// Per-word addition is associative and `empty()` is its identity; under
/// `contracts = "test"` the `Monoid` laws are property-tested for this impl.
impl Monoid for Counts {}

/// Authority-free and parallel. Equals the sequential fold over `docs`.
pub fn word_freq(docs: &[String]) -> Counts {
    docs.par_map(|doc| Counts::of(doc)).par_reduce()
}

#[derive(Debug, Error)]
enum AppError {
    #[error("cannot read {path}")]
    Read { path: String, source: io::Error },
}

/// Authority: reads the named files through `fs`.
fn read_all(fs: &Fs, paths: &[String]) -> Result<Vec<String>, AppError> {
    paths
        .iter()
        .map(|path| {
            fs.read_to_string(path)
                .map_err(|source| AppError::Read { path: path.clone(), source })
        })
        .collect()
}

fn main(root: Root) -> Result<(), AppError> {
    let paths: Vec<String> = root.env.args().skip(1).collect();
    let docs = read_all(&root.fs.read_only(), &paths)?; // I/O first, sequentially
    let counts = word_freq(&docs);                       // then pure, parallel work
    for (word, n) in counts.top(TOP) {
        println!("{n:>8} {word}");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn counts(pairs: &[(&str, u64)]) -> Counts {
        Counts(pairs.iter().map(|&(w, n)| (w.to_string(), n)).collect())
    }

    #[test]
    fn counts_one_document() {
        let got = Counts::of("The cat and THE hat; the end.");
        assert_eq!(got, counts(&[("and", 1), ("cat", 1), ("end", 1), ("hat", 1), ("the", 3)]));
    }

    #[test]
    fn merges_across_documents() {
        let docs = vec!["a b".to_string(), "b c".to_string(), String::new(), "c c".to_string()];
        assert_eq!(word_freq(&docs), counts(&[("a", 1), ("b", 2), ("c", 3)]));
        assert_eq!(word_freq(&[]), Counts::empty());
    }

    #[test]
    fn top_breaks_ties_alphabetically() {
        let c = counts(&[("pear", 2), ("apple", 2), ("fig", 5), ("kiwi", 1)]);
        assert_eq!(c.top(3), vec![("fig", 5), ("apple", 2), ("pear", 2)]);
    }

    #[test]
    fn parallel_equals_sequential(docs: Vec<String>) {
        let sequential = docs.iter().fold(Counts::empty(), |acc, doc| acc.combine(&Counts::of(doc)));
        assert_eq!(word_freq(&docs), sequential);
    }
}
