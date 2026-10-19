// src/main.fx of package `trials/wc`
//
// `wc FILE...` prints line, word, and byte counts for each file and a total.
// A file that cannot be read prints an error on stderr; the exit code is then 1.

use std::fs::Fs;
use std::io::{self, Read};
use std::ops::AddAssign;
use std::process::ExitCode;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Counts {
    pub lines: u64,
    pub words: u64,
    pub bytes: u64,
}

impl AddAssign for Counts {
    fn add_assign(&mut self, other: Counts) {
        self.lines += other.lines;
        self.words += other.words;
        self.bytes += other.bytes;
    }
}

/// Streaming counter. A word is a maximal run of non-whitespace bytes, as in
/// POSIX `wc`; `in_word` carries that state across chunk boundaries.
#[derive(Debug, Default)]
struct Counter {
    counts: Counts,
    in_word: bool,
}

impl Counter {
    fn feed(&mut self, chunk: &[u8]) {
        for &b in chunk {
            if b == b'\n' {
                self.counts.lines += 1;
            }
            let space = b.is_ascii_whitespace();
            if !space && !self.in_word {
                self.counts.words += 1;
            }
            self.in_word = !space;
        }
        self.counts.bytes += chunk.len() as u64; // usize -> u64 is lossless
    }
}

/// Authority-free.
#[pure]
pub fn count_bytes(data: &[u8]) -> Counts {
    let mut c = Counter::default();
    c.feed(data);
    c.counts
}

/// Authority: reads one file through `fs`.
pub fn count_file(fs: &Fs, path: &str) -> io::Result<Counts> {
    let mut file = fs.open(path)?;
    let mut buf = vec![0u8; 64 * 1024];
    let mut counter = Counter::default();
    loop {
        let n = match file.read(&mut buf) {
            Ok(0) => return Ok(counter.counts),
            Ok(n) => n,
            Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
            Err(e) => return Err(e),
        };
        counter.feed(&buf[..n]);
    }
}

pub struct Report {
    pub out: String,
    pub err: String,
    pub ok: bool,
}

fn row(c: &Counts, name: &str) -> String {
    format!("{:>8} {:>8} {:>8} {name}\n", c.lines, c.words, c.bytes)
}

/// Authority: reads the named files through `fs`, nothing else.
pub fn run(fs: &Fs, paths: &[String]) -> Report {
    let mut report = Report { out: String::new(), err: String::new(), ok: true };
    let mut total = Counts::default();
    for path in paths {
        match count_file(fs, path) {
            Ok(c) => {
                report.out.push_str(&row(&c, path));
                total += c;
            }
            Err(e) => {
                report.err.push_str(&format!("wc: {path}: {e}\n"));
                report.ok = false;
            }
        }
    }
    report.out.push_str(&row(&total, "total"));
    report
}

fn main(root: Root) -> ExitCode {
    let paths: Vec<String> = root.env.args().skip(1).collect();
    if paths.is_empty() {
        eprintln!("usage: wc FILE...");
        return ExitCode::from(2);
    }
    // wc never writes; paths may be absolute, so attenuate by mode, not by directory.
    let fs = root.fs.read_only();
    let report = run(&fs, &paths);
    print!("{}", report.out);
    eprint!("{}", report.err);
    if report.ok { ExitCode::SUCCESS } else { ExitCode::FAILURE }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::Mode;

    #[test]
    fn counts_like_wc() {
        const CASES: [(&str, Counts); 5] = [
            ("", Counts { lines: 0, words: 0, bytes: 0 }),
            ("one\n", Counts { lines: 1, words: 1, bytes: 4 }),
            ("a b\tc\n\nd", Counts { lines: 2, words: 4, bytes: 8 }),
            ("  lead and trail  ", Counts { lines: 0, words: 3, bytes: 18 }),
            ("héllo wörld\n", Counts { lines: 1, words: 2, bytes: 14 }),
        ];
        for (text, want) in CASES {
            assert_eq!(count_bytes(text.as_bytes()), want, "input {text:?}");
        }
    }

    #[test]
    fn chunking_does_not_change_counts(text: String, split: usize) {
        let bytes = text.as_bytes();
        let at = split % (bytes.len() + 1);
        let mut c = Counter::default();
        c.feed(&bytes[..at]);
        c.feed(&bytes[at..]);
        assert_eq!(c.counts, count_bytes(bytes));
    }

    #[test]
    fn missing_file_is_reported_and_fails() {
        let sim = Root::sim(1);
        sim.world.file("/w/a.txt", "one two\nthree\n");
        sim.world.file("/w/b.txt", "four\n");
        let fs = sim.fs.dir("/w", Mode::Read).unwrap();
        let paths: Vec<String> = ["a.txt", "missing.txt", "b.txt"].iter().map(|s| s.to_string()).collect();

        let report = run(&fs, &paths);

        assert!(!report.ok);
        assert!(report.err.starts_with("wc: missing.txt: "));
        assert_eq!(
            report.out,
            "       2        3       14 a.txt\n       1        1        5 b.txt\n       3        4       19 total\n"
        );
        assert_eq!(count_file(&fs, "missing.txt").unwrap_err().kind(), io::ErrorKind::NotFound);
    }
}
