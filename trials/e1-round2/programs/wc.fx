//! `wc FILE...`: line, word, and byte counts per file, then a total.
//! A file that cannot be read prints an error to stderr, the other files are
//! still counted, and the exit code is 1. No arguments: usage, exit code 2.

use std::fs::Fs;
use std::io;
use std::process::ExitCode;

/// Line, word, and byte counts of one input.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Counts {
    pub lines: usize,
    pub words: usize,
    pub bytes: usize,
}

impl Counts {
    /// Authority-free. Lines are `\n` bytes; words are maximal runs of
    /// non-whitespace bytes, as POSIX `wc` counts them in the C locale.
    pub fn of(data: &[u8]) -> Counts {
        Counts {
            lines: data.iter().filter(|&&b| b == b'\n').count(),
            words: data.split(|b| b.is_ascii_whitespace()).filter(|w| !w.is_empty()).count(),
            bytes: data.len(),
        }
    }

    pub fn add(&mut self, other: Counts) {
        self.lines += other.lines;
        self.words += other.words;
        self.bytes += other.bytes;
    }

    /// One output row, right-aligned like `wc`.
    pub fn row(&self, name: &str) -> String {
        format!("{:>8} {:>8} {:>8} {name}", self.lines, self.words, self.bytes)
    }
}

#[derive(Debug, Error)]
pub enum WcError {
    #[error("{path}: {source}")]
    Read { path: String, source: io::Error },
}

/// Authority: reads one file through `fs`, nothing else.
pub fn count_file(fs: &Fs, path: &str) -> Result<Counts, WcError> {
    let data = fs.read(path).map_err(|source| WcError::Read { path: path.to_string(), source })?;
    Ok(Counts::of(&data))
}

fn main(root: Root) -> ExitCode {
    let paths: Vec<String> = root.env.args().skip(1).collect();   // args[0] is the program name
    if paths.is_empty() {
        eprintln!("usage: wc FILE...");
        return ExitCode::from(2u8);
    }
    let fs = root.fs.read_only();
    let mut total = Counts::default();
    let mut failed = false;
    for path in &paths {
        match count_file(&fs, path) {
            Ok(counts) => {
                println!("{}", counts.row(path));
                total.add(counts);
            }
            Err(e) => {
                eprintln!("wc: {e}");
                failed = true;
            }
        }
    }
    println!("{}", total.row("total"));
    if failed { ExitCode::FAILURE } else { ExitCode::SUCCESS }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::Mode;

    #[test]
    fn counts_lines_words_and_bytes() {
        assert_eq!(Counts::of(b""), Counts::default());
        assert_eq!(Counts::of(b"one two\n  three\n"), Counts { lines: 2, words: 3, bytes: 16 });
        assert_eq!(Counts::of(b"no newline"), Counts { lines: 0, words: 2, bytes: 10 });
        assert_eq!(Counts { lines: 1, words: 2, bytes: 3 }.row("x"), "       1        2        3 x");
    }

    #[test]
    fn reads_files_and_reports_missing_ones() {
        let sim = Root::sim(1);
        sim.world.file("/w/a.txt", "a b\nc\n");
        let fs = sim.fs.dir("/w", Mode::Read).unwrap();
        assert_eq!(count_file(&fs, "a.txt").unwrap(), Counts { lines: 2, words: 3, bytes: 6 });
        match count_file(&fs, "nope.txt") {
            Err(WcError::Read { path, source }) => {
                assert_eq!(path, "nope.txt");
                assert_eq!(source.kind(), io::ErrorKind::NotFound);
            }
            other => panic!("expected NotFound, got {other:?}"),
        }
    }

    #[test]
    fn lines_and_bytes_add_up_over_concatenation(a: Vec<u8>, b: Vec<u8>) {
        let mut both = a.clone();
        both.extend_from_slice(&b);
        let (x, y, z) = (Counts::of(&a), Counts::of(&b), Counts::of(&both));
        assert_eq!(z.lines, x.lines + y.lines);
        assert_eq!(z.bytes, x.bytes + y.bytes);
        assert!(z.words <= x.words + y.words);         // a word may straddle the join
    }
}
