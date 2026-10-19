//! `word_freq FILE...`: the 20 most frequent words across the files.

use std::collections::{BTreeMap, HashMap};
use std::fs::Fs;
use std::io;

/// Authority-free. Words are maximal runs of alphanumeric characters,
/// lowercased.
pub fn count_words(doc: &str) -> HashMap<String, u64> {
    let mut counts = HashMap::new();
    for word in doc.split(|c: char| !c.is_alphanumeric()).filter(|w| !w.is_empty()) {
        *counts.entry(word.to_lowercase()).or_insert(0) += 1;
    }
    counts
}

/// Counts each document in parallel, then merges the per-document counts
/// sequentially, in document order, into a sorted map. The result equals
/// the sequential computation for every schedule.
#[pure]
pub fn word_freq(docs: &[String]) -> BTreeMap<String, u64> {
    let per_doc: Vec<HashMap<String, u64>> = docs.par_map(|doc| count_words(doc));
    let mut total = BTreeMap::new();
    for counts in per_doc {
        for (word, n) in counts {
            *total.entry(word).or_insert(0) += n;
        }
    }
    total
}

/// The `k` most frequent words, most frequent first; ties in word order.
pub fn top(freq: &BTreeMap<String, u64>, k: usize) -> Vec<(&str, u64)> {
    let mut ranked: Vec<(&str, u64)> = freq.iter().map(|(w, &n)| (w.as_str(), n)).collect();
    ranked.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(b.0)));
    ranked.truncate(k);
    ranked
}

/// Reads every file, sequentially: `par_map` closures cannot do I/O.
fn read_all(fs: &Fs, paths: &[String]) -> io::Result<Vec<String>> {
    paths.iter().map(|p| fs.read_to_string(p)).collect()
}

fn main(root: Root) -> Result<(), io::Error> {
    let fs = root.fs.read_only();
    let paths: Vec<String> = root.env.args().skip(1).collect();
    let docs = read_all(&fs, &paths)?;
    for (word, n) in top(&word_freq(&docs), 20) {
        println!("{n:>8} {word}");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sequential(docs: &[String]) -> BTreeMap<String, u64> {
        let mut total = BTreeMap::new();
        for doc in docs {
            for (word, n) in count_words(doc) {
                *total.entry(word).or_insert(0) += n;
            }
        }
        total
    }

    #[test]
    fn counts_across_documents() {
        let docs = vec![String::from("The cat. the HAT!"), String::from("a cat, a hat; the end")];
        let freq = word_freq(&docs);
        assert_eq!((freq["the"], freq["cat"], freq["end"]), (3, 2, 1));
        assert_eq!(top(&freq, 3), vec![("the", 3), ("a", 2), ("cat", 2)]);
    }

    #[test]
    fn parallel_equals_sequential(docs: Vec<String>) {
        assert_eq!(word_freq(&docs), sequential(&docs));
    }

    #[test]
    fn document_order_does_not_matter(mut docs: Vec<String>) {
        let forward = word_freq(&docs);
        docs.reverse();
        assert_eq!(word_freq(&docs), forward);
    }
}
