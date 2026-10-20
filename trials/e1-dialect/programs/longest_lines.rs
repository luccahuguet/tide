// Task 4: longest_lines -- src/lib.rs
//
// One child task per document. Children borrow the documents directly: they
// cannot outlive the scope, and the scope cannot outlive the borrow.

use std::task::{TaskError, Tasks};

#[derive(Debug, Error, PartialEq)]
pub enum DocError {
    #[error("document {0} contains a NUL byte")]
    Nul(usize),
    #[error("task failed: {0}")]
    Task(#[from] TaskError),
}

/// For each document in order, its longest line (by characters; the first on
/// ties) as a slice of that document, `""` for an empty one. At most
/// `max_parallel` children run at once.
#[requires(max_parallel > 0)]
pub fn longest_lines<'d>(tasks: &Tasks, docs: &[&'d str], max_parallel: usize)
    -> Result<Vec<&'d str>, DocError>
{
    tasks.limit(max_parallel).scope(|s| {
        let handles: Vec<_> = docs
            .iter()
            .enumerate()
            .map(|(i, doc)| {
                let doc: &'d str = doc;
                s.spawn(move || longest_line(i, doc))
            })
            .collect();
        handles.into_iter().map(|h| h.join()).collect()
    })
}

/// The number of whitespace-separated words across all documents, one child
/// per document.
pub fn total_words(tasks: &Tasks, docs: &[String]) -> Result<usize, DocError> {
    tasks.scope(|s| {
        let handles: Vec<_> = docs
            .iter()
            .enumerate()
            .map(|(i, doc)| s.spawn(move || count_words(i, doc)))
            .collect();
        let mut total = 0;
        for h in handles {
            total += h.join()?;
        }
        Ok(total)
    })
}

fn check_nul(index: usize, doc: &str) -> Result<(), DocError> {
    if doc.contains('\0') { Err(DocError::Nul(index)) } else { Ok(()) }
}

fn longest_line(index: usize, doc: &str) -> Result<&str, DocError> {
    check_nul(index, doc)?;
    let mut best = &doc[..0]; // a slice of `doc` even when `doc` has no lines
    let mut best_len = 0;
    for line in doc.lines() {
        let len = line.chars().count();
        if len > best_len {
            best = line;
            best_len = len;
        }
    }
    Ok(best)
}

fn count_words(index: usize, doc: &str) -> Result<usize, DocError> {
    check_nul(index, doc)?;
    Ok(doc.split_whitespace().count())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sim::SimRoot;

    #[sim_test]
    fn three_documents(sim: SimRoot) {
        let docs = ["short\nthe longest line\nmid", "", "a\r\nbb\r\ncc"];
        let got = longest_lines(&sim.tasks, &docs, 2).unwrap();
        assert_eq!(got, vec!["the longest line", "", "bb"]);

        let owned: Vec<String> = vec!["a b c".into(), "".into(), "d  e\nf".into()];
        assert_eq!(total_words(&sim.tasks, &owned), Ok(6));
    }

    #[sim_test]
    fn nul_document_fails_the_whole_call(sim: SimRoot) {
        let docs = ["fine", "bad\0doc", "also fine"];
        assert_eq!(longest_lines(&sim.tasks, &docs, 1), Err(DocError::Nul(1)));
        assert_eq!(longest_lines(&sim.tasks, &docs, 3), Err(DocError::Nul(1)));

        let owned: Vec<String> = docs.iter().map(|d| d.to_string()).collect();
        assert_eq!(total_words(&sim.tasks, &owned), Err(DocError::Nul(1)));
    }

    #[sim_test]
    fn result_outlives_the_docs_vec(sim: SimRoot) {
        let a = String::from("x\nlonger");
        let b = String::from("one\ntwo three");
        let got;
        {
            let docs: Vec<&str> = vec![a.as_str(), b.as_str()];
            got = longest_lines(&sim.tasks, &docs, 1).unwrap();
        }
        // `docs` is gone; `a` and `b` are alive.
        assert_eq!(got, vec!["longer", "two three"]);
    }
}
