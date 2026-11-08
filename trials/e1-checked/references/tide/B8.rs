use std::json;
use std::{collections::BTreeMap, process::ExitCode};
#[derive(Decode)]
#[codec(rename_all = "lowercase")]
enum Op {
    Read,
    Write,
}
#[derive(Decode)]
struct Record {
    user: String,
    bytes: u64,
    op: Op,
}
fn run(stdin: &std::io::Stdin) -> u8 {
    let mut text = String::new();
    if let Err(e) = stdin.read_to_string(&mut text) {
        eprintln!("usage: {e}");
        return 1;
    }
    let mut totals = BTreeMap::<String, (u64, u64)>::new();
    let mut code = 0;
    for (i, line) in text
        .lines()
        .enumerate()
        .filter(|(_, s)| !s.trim().is_empty())
    {
        match json::decode::<Record>(line.as_bytes()) {
            Ok(r) => {
                let entry = totals.entry(r.user).or_default();
                let n = match r.op {
                    Op::Read => &mut entry.0,
                    Op::Write => &mut entry.1,
                };
                match n.checked_add(r.bytes) {
                    Some(sum) => *n = sum,
                    None => {
                        eprintln!("usage: overflow");
                        return 1;
                    }
                }
            }
            Err(e) => {
                eprintln!("usage: line {}: {e}", i + 1);
                code = 1;
            }
        }
    }
    for (user, (r, w)) in totals {
        println!("{user} read={r} write={w}");
    }
    code
}
#[tide::main]
fn main(root: Root) -> ExitCode {
    ExitCode::from(run(&root.stdin))
}
