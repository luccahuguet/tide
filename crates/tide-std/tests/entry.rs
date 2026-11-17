//! Programs built against Tide's std injected as `std` (`--extern std=`), as the admission driver
//! injects it, and run: the entry `#[tide::main]` generates, what the sinks write, and what a
//! program's result and panics do to its exit status. rustc alone builds them; admission is the
//! driver's, which the conformance runner (`crates/tide-conformance`) checks.

use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::sync::OnceLock;

mod support;

fn tmp() -> PathBuf {
    PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("entry")
}

/// The toolchain cargo built this test with, as rustup resolves it in this package.
fn rustc() -> Command {
    let mut c = Command::new(std::env::var_os("RUSTC").unwrap_or("rustc".into()));
    c.current_dir(env!("CARGO_MANIFEST_DIR"))
        .args(["--edition=2024", "--error-format=short"]);
    c
}

/// Builds `tide_macros` and `tide_std` into one directory, once per test process.
fn std_dir() -> &'static Path {
    static DIR: OnceLock<PathBuf> = OnceLock::new();
    DIR.get_or_init(|| {
        let dir = tmp().join("std");
        support::build_std(&dir)
    })
}

/// Compiles `source` as a binary against Tide's std; returns rustc's output and the binary.
fn compile(name: &str, source: &str) -> (Output, PathBuf) {
    let std = std_dir();
    let file = tmp().join(format!("{name}.rs"));
    std::fs::write(&file, source).unwrap();
    let exe = tmp().join(name);
    let out = rustc()
        .args(["--crate-type=bin", "--crate-name", name])
        .arg(format!(
            "--extern=std={}",
            std.join("libtide_std.rlib").display()
        ))
        .arg(format!("-Ldependency={}", std.display()))
        .arg("-o")
        .arg(&exe)
        .arg(&file)
        .stdin(Stdio::null())
        .output()
        .unwrap();
    (out, exe)
}

/// Builds `source` and runs it; returns its exit code, stdout, and stderr.
fn exec(name: &str, source: &str) -> (Option<i32>, String, String) {
    let (out, exe) = compile(name, source);
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let out = Command::new(exe).stdin(Stdio::null()).output().unwrap();
    let text = |b: Vec<u8>| String::from_utf8(b).unwrap();
    (out.status.code(), text(out.stdout), text(out.stderr))
}

/// rustc's short-format errors for `source`, which must not compile.
fn rejected(name: &str, source: &str) -> String {
    let (out, _) = compile(name, source);
    assert!(!out.status.success(), "{name} compiled");
    let stderr = String::from_utf8(out.stderr).unwrap();
    stderr
        .lines()
        .filter(|l| l.contains(": error"))
        .map(|l| {
            l.split_once(&format!("{name}.rs:"))
                .map_or(l, |(_, rest)| rest)
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// A record is formatted whole before any of it is written (SINK-3): a closed stderr does not
/// skip `Loud`'s formatting, and the records `Loud` prints come before the one it is part of.
#[test]
fn records_are_formatted_before_they_are_written() {
    let source = r#"
struct Loud;
impl std::fmt::Display for Loud {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        println!("formatted");
        f.write_str("x")
    }
}

#[tide::main]
fn main() {
    eprintln!("prefix {}{}", Loud, Loud);
    println!("outer[{}]", Loud);
}
"#;
    let (out, exe) = compile("formatted", source);
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let mut child = Command::new(exe)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    drop(child.stderr.take());
    let out = child.wait_with_output().unwrap();
    assert_eq!(
        (out.status.code(), String::from_utf8(out.stdout).unwrap()),
        (
            Some(0),
            "formatted\nformatted\nformatted\nouter[x]\n".into()
        )
    );
    // As in Rust, a formatting impl that returns an error panics, and nothing is written.
    let source = r#"
struct Fails;
impl std::fmt::Display for Fails {
    fn fmt(&self, _: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Err(std::fmt::Error)
    }
}

#[tide::main]
fn main() {
    println!("head {}", Fails);
}
"#;
    let (status, out, err) = exec("fails", source);
    assert_eq!((status, out.as_str()), (Some(101), ""));
    assert!(err.contains("formatting trait implementation returned an error"), "{err}");
    // As in Rust, only the line forms take no arguments.
    let errors = rejected("bare", "#[tide::main]\nfn main() {\n    print!();\n}\n");
    assert!(errors.starts_with("3:5: error"), "{errors}");
}

#[test]
fn main_receives_root_and_the_sinks_write_whole_records_in_order() {
    let source = r#"
#[tide::main]
fn main(root: Root) {
    let _net: std::net::Net = root.net;
    let tasks: &std::task::Tasks = &root.tasks;
    let _ = tasks.clone();
    print!("a");
    println!("b {}", 1);
    eprint!("c");
    eprintln!("d{}", "e");
    println!();
    eprintln!();
}
"#;
    assert_eq!(
        exec("with_root", source),
        (Some(0), "ab 1\n\n".into(), "cde\n\n".into())
    );
}

#[test]
fn main_reports_its_result_as_rust_does() {
    let failed = "#[tide::main]\nfn main() -> Result<(), String> {\n    Err(\"bad\".into())\n}\n";
    assert_eq!(
        exec("failed", failed),
        (Some(1), String::new(), "Error: \"bad\"\n".into())
    );
    let code = "#[tide::main]\nfn main(_root: Root) -> std::process::ExitCode {\n    std::process::ExitCode::from(7)\n}\n";
    assert_eq!(exec("code", code), (Some(7), String::new(), String::new()));
    let (status, out, err) = exec(
        "panics",
        "#[tide::main]\nfn main() {\n    panic!(\"boom {}\", 1);\n}\n",
    );
    assert_eq!((status, out.as_str()), (Some(101), ""));
    assert!(err.contains("boom 1"), "{err}");
}

/// A closed stdout ends the program quietly with status 141 (SINK-5); `err` writes nothing
/// after it, so the loop did not run on.
#[test]
fn a_closed_stdout_exits_with_141() {
    let source = r#"
#[tide::main]
fn main() {
    for i in 0..100_000 {
        println!("{i:>80}");
    }
    eprintln!("finished");
}
"#;
    let (out, exe) = compile("closed", source);
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let mut child = Command::new(exe)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    drop(child.stdout.take());
    let out = child.wait_with_output().unwrap();
    assert_eq!(
        (out.status.code(), String::from_utf8(out.stderr).unwrap()),
        (Some(141), String::new())
    );
}

/// Only the runtime entry mints a `Root` or a capability, and it runs once per process: a
/// program that calls it again, which the admission driver rejects (`TD0520`), panics before
/// minting.
#[test]
fn root_is_minted_only_by_the_entry_once() {
    let again = "#[tide::main]\nfn main() {\n    let _ = std::__rt::enter(|_root| ());\n}\n";
    let (status, out, err) = exec("again", again);
    assert_eq!((status, out.as_str()), (Some(101), ""));
    assert!(
        err.contains("the runtime entry runs once per process"),
        "{err}"
    );

    // Neither a `Root` nor a capability is made from data (ROOT-4).
    for (name, body, code) in [
        (
            "copied",
            "let _copy = Root { ..root };",
            "3:26: error[E0451]",
        ),
        ("built", "let _fs = std::fs::Fs(());", "3:24: error[E0603]"),
        (
            "defaulted",
            "let _net: std::net::Net = Default::default();",
            "3:31: error[E0277]",
        ),
    ] {
        let source =
            format!("#[tide::main]\nfn main(root: Root) {{\n    {body}\n    let _ = root;\n}}\n");
        let errors = rejected(name, &source);
        assert!(errors.starts_with(code), "{code} in:\n{errors}");
    }
}

#[test]
fn the_attribute_goes_on_main_without_arguments() {
    let errors = rejected("misplaced", "#[tide::main]\nfn start() {}\nfn main() {}\n");
    assert!(
        errors.contains("2:4: error: `#[tide::main]` goes on the binary's `fn main`"),
        "{errors}"
    );
    let errors = rejected("arguments", "#[tide::main(fast)]\nfn main() {}\n");
    assert!(
        errors.contains("1:14: error: `#[tide::main]` takes no arguments"),
        "{errors}"
    );
}
