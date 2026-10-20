// Task 11: dedupe -- src/main.rs
//
// dedupe [--count] [FILE]: prints each distinct line of FILE (or standard
// input) once, in order of first appearance; with --count, each line is
// prefixed by its number of occurrences and a tab. Exit codes: 0 on success,
// 2 on a usage error, 1 if the input cannot be read.

use std::collections::HashMap;
use std::env::Env;
use std::fs::Fs;
use std::io::{self, Stdin};
use std::process::ExitCode;

#[derive(Debug, Error)]
enum CliError {
    #[error("{0}\nusage: dedupe [--count] [FILE]")]
    Usage(String),
    #[error("cannot read {path}: {source}")]
    Read { path: String, source: io::Error },
    #[error("cannot read standard input: {source}")]
    Stdin { source: io::Error },
}

impl CliError {
    fn exit_code(&self) -> u8 {
        match self {
            CliError::Usage(_) => 2,
            CliError::Read { .. } | CliError::Stdin { .. } => 1,
        }
    }
}

#[derive(Debug, PartialEq)]
struct Options {
    count: bool,
    file: Option<String>,
}

/// Authority-free. `args[0]` is the program name.
fn parse_args(args: &[String]) -> Result<Options, CliError> {
    let mut opts = Options { count: false, file: None };
    for arg in args.iter().skip(1) {
        match arg.as_str() {
            "--count" => opts.count = true,
            s if s.starts_with('-') => {
                return Err(CliError::Usage(format!("unknown flag: {s}")));
            }
            s => {
                if opts.file.is_some() {
                    return Err(CliError::Usage("more than one file given".to_string()));
                }
                opts.file = Some(s.to_string());
            }
        }
    }
    Ok(opts)
}

/// Authority-free: the distinct lines of `text` in order of first appearance,
/// each ending in `\n`, prefixed by its count and a tab when `count` is set.
fn dedupe(text: &str, count: bool) -> String {
    let mut first_seen: Vec<&str> = Vec::new();
    let mut counts: HashMap<&str, u64> = HashMap::new();
    for line in text.lines() {
        let c = counts.entry(line).or_insert(0);
        if *c == 0 {
            first_seen.push(line);
        }
        *c += 1;
    }
    let mut out = String::new();
    for line in first_seen {
        if count {
            out.push_str(&format!("{}\t", counts[line]));
        }
        out.push_str(line);
        out.push('\n');
    }
    out
}

/// The whole program except printing: arguments from `env`, input from `fs`
/// or `stdin`.
fn run(env: &Env, fs: &Fs, stdin: &Stdin) -> Result<String, CliError> {
    let args: Vec<String> = env.args().collect();
    let opts = parse_args(&args)?;
    let text = match &opts.file {
        Some(path) => fs
            .read_to_string(path)
            .map_err(|source| CliError::Read { path: path.clone(), source })?,
        None => read_all(stdin).map_err(|source| CliError::Stdin { source })?,
    };
    Ok(dedupe(&text, opts.count))
}

fn read_all(stdin: &Stdin) -> io::Result<String> {
    let mut text = String::new();
    while stdin.read_line(&mut text)? > 0 {}
    Ok(text)
}

#[tide::main]
fn main(root: Root) -> ExitCode {
    let fs = root.fs.read_only();
    match run(&root.env, &fs, &root.stdin) {
        Ok(out) => {
            print!("{out}");
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("dedupe: {e}");
            ExitCode::from(e.exit_code())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sim::SimRoot;

    fn args(v: &[&str]) -> Vec<String> {
        v.iter().map(|a| a.to_string()).collect()
    }

    #[test]
    fn repeated_lines_without_counts() {
        assert_eq!(dedupe("b\na\nb\nc\na\n", false), "b\na\nc\n");
        assert_eq!(dedupe("x\r\nx\ny", false), "x\ny\n");
        assert_eq!(dedupe("", false), "");
    }

    #[test]
    fn repeated_lines_with_counts() {
        assert_eq!(dedupe("b\na\nb\nc\na\nb\n", true), "3\tb\n2\ta\n1\tc\n");
        assert_eq!(dedupe("\n\n", true), "2\t\n");
    }

    #[test]
    fn usage_errors() {
        assert_eq!(parse_args(&args(&["dedupe"])).unwrap(), Options { count: false, file: None });
        assert_eq!(
            parse_args(&args(&["dedupe", "f.txt", "--count"])).unwrap(),
            Options { count: true, file: Some("f.txt".to_string()) }
        );
        for bad in [&["dedupe", "--bogus"][..], &["dedupe", "a.txt", "b.txt"][..], &["dedupe", "-c"][..]] {
            let err = parse_args(&args(bad)).unwrap_err();
            assert!(matches!(err, CliError::Usage(_)));
            assert_eq!(err.exit_code(), 2);
        }
    }

    #[sim_test(seeds = 10)]
    fn whole_program_reads_a_named_file(sim: SimRoot) {
        sim.world.file("/work/in.txt", "b\na\nb\nc\na\n");
        sim.world.args(["dedupe", "--count", "/work/in.txt"]);
        assert_eq!(run(&sim.env, &sim.fs, &sim.stdin).unwrap(), "2\tb\n2\ta\n1\tc\n");
    }

    #[sim_test(seeds = 10)]
    fn whole_program_reads_standard_input(sim: SimRoot) {
        sim.world.stdin("x\ny\nx\nz");
        sim.world.args(["dedupe"]);
        assert_eq!(run(&sim.env, &sim.fs, &sim.stdin).unwrap(), "x\ny\nz\n");
    }

    #[sim_test(seeds = 10)]
    fn a_missing_file_exits_with_one(sim: SimRoot) {
        sim.world.args(["dedupe", "/work/missing.txt"]);
        let err = run(&sim.env, &sim.fs, &sim.stdin).unwrap_err();
        assert!(matches!(&err, CliError::Read { source, .. } if source.kind() == io::ErrorKind::NotFound));
        assert_eq!(err.exit_code(), 1);
    }
}
