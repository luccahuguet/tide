//! Reviewed demo programs built through the driver against production std; only this trusted
//! test caller executes them. This is not the RUN-3 sandbox or Tide's test runtime.
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

#[path = "support/admitted.rs"]
mod admitted;
use admitted::{checked, driver, package, plain_rustc, std_dir, stderr};

const HELLO: &str = include_str!("../../../conformance/cases/authority/root_executable.rs");

mod reference_allowlist {
    include!("support/reference_allowlist.rs");
}

#[test]
fn reference_kept_families_build_and_execute_against_production_std() {
    let mut source = reference_allowlist::source();
    source.push_str(
        r#"
#[tide::main]
fn main(root: Root) {
    run();
    // §1's replacements are Tide-owned rather than core/alloc allowlist entries.
    let mut map = std::collections::HashMap::new(); map.insert("a", 1);
    let mut set = std::collections::HashSet::new(); set.insert("a");
    assert_eq!(map["a"], 1); assert!(set.contains("a"));
    let now = root.clock.now(); let wall = root.clock.system_time();
    assert_eq!(now.duration_since(now), std::time::Duration::ZERO);
    assert_eq!(wall.duration_since(wall).unwrap(), std::time::Duration::ZERO);
    {
        use std::io::{Read, Write, Seek, BufRead};
        let mut c = std::io::Cursor::new(vec![0; 3]);
        c.write_all(b"abc").unwrap(); c.seek(std::io::SeekFrom::Start(0)).unwrap();
        let mut r = std::io::BufReader::new(c);
        assert_eq!(r.fill_buf().unwrap(), b"abc");
        let mut s = String::new(); r.read_to_string(&mut s).unwrap(); assert_eq!(s, "abc");
        let mut w = std::io::BufWriter::new(Vec::new());
        w.write_all(b"abc").unwrap(); w.flush().unwrap();
        assert_eq!(w.into_inner().unwrap(), b"abc");
        let e: std::io::Result<()> = Err(std::io::ErrorKind::NotFound.into());
        assert_eq!(e.unwrap_err().kind(), std::io::ErrorKind::NotFound);
    }
    println!("REFERENCE_ALLOWLIST_RAN");
}
"#,
    );
    let dir = package("reference-allowlist", &source);
    let build = checked(&mut driver(&dir));
    assert!(!stderr(&build).contains("TD0520:"), "{}", stderr(&build));
    let out = checked(&mut Command::new(dir.join("out/demo")));
    assert_eq!(out.stdout, b"REFERENCE_ALLOWLIST_RAN\n");
    assert!(out.stderr.is_empty(), "{}", stderr(&out));
}

#[test]
fn reference_io_error_last_os_error_stays_denied() {
    // Rust's io::Error has a different key and is not Tide's retained error type.
    let dir = package(
        "reference-deny-last-os-error",
        r#"
#[tide::main]
fn main() { let _ = std::io::Error::last_os_error(); }
"#,
    );
    let out = driver(&dir)
        .arg("--emit=metadata")
        .stdin(Stdio::null())
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1), "{}", stderr(&out));
    assert!(
        stderr(&out)
            .contains("error: TD0520: `tide_std::io::Error::last_os_error` is not admitted"),
        "{}",
        stderr(&out)
    );
}

/// One body, built through the driver against production std and with plain rustc against
/// Rust's std: `Read` and `Write` for `VecDeque<u8>` print the same transcript in both.
#[test]
fn vec_deque_reads_and_writes_as_rusts_std() {
    const BODY: &str = r#"
use std::collections::VecDeque;
use std::io::{Read, Write};

/// "abcdef" stored wrapped: the front slice "ab" ends the buffer, the back slice "cdef" starts it.
fn ring() -> VecDeque<u8> {
    let mut q = VecDeque::with_capacity(8);
    q.extend(*b"cdef");
    q.push_front(b'b');
    q.push_front(b'a');
    assert_eq!(q.as_slices(), (&b"ab"[..], &b"cdef"[..]));
    q
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

fn rest(q: &VecDeque<u8>) -> String {
    text(&q.iter().copied().collect::<Vec<u8>>())
}

fn res<T: std::fmt::Debug>(r: std::io::Result<T>) -> String {
    match r {
        Ok(v) => format!("Ok({v:?})"),
        Err(e) => format!("Err({:?}: {e})", e.kind()),
    }
}

fn run() {
    // `read` drains the front slice only, then the back.
    let mut q = ring();
    let mut buf = [0u8; 8];
    let r = q.read(&mut buf);
    println!("read wrapped: {} {:?} rest {:?}", res(r), text(&buf), rest(&q));
    let r = q.read(&mut buf[..3]);
    println!("read short: {} {:?} rest {:?}", res(r), text(&buf[..3]), rest(&q));
    let r = q.read(&mut []);
    println!("read none: {} rest {:?}", res(r), rest(&q));

    // `read_exact` across the slice boundary, at it, and one past the end.
    for n in [0, 1, 2, 3, 6, 7] {
        let mut q = ring();
        let mut buf = vec![b'.'; n];
        let r = q.read_exact(&mut buf);
        println!("read_exact {n} wrapped: {} {:?} rest {:?}", res(r), text(&buf), rest(&q));
    }
    for n in [3, 4] {
        let mut q = VecDeque::from(b"abc".to_vec());
        let mut buf = vec![b'.'; n];
        let r = q.read_exact(&mut buf);
        println!("read_exact {n} flat: {} {:?} rest {:?}", res(r), text(&buf), rest(&q));
    }

    // Whole reads take both slices.
    let mut q = ring();
    let mut v = b">".to_vec();
    let r = q.read_to_end(&mut v);
    println!("read_to_end: {} {:?} rest {:?}", res(r), text(&v), rest(&q));
    let mut q = ring();
    let mut s = String::from(">");
    let r = q.read_to_string(&mut s);
    println!("read_to_string: {} {s:?} rest {:?}", res(r), rest(&q));
    let mut q = VecDeque::from(vec![b'x', 0xff]);
    let mut s = String::from(">");
    let r = q.read_to_string(&mut s);
    println!("read_to_string invalid: {} {s:?} rest {:?}", res(r), rest(&q));

    // Provided methods over the impl.
    let mut q = ring();
    let mut v = Vec::new();
    let r = (&mut q).take(4).read_to_end(&mut v);
    println!("take 4: {} {:?} rest {:?}", res(r), text(&v), rest(&q));
    let bytes: Vec<u8> = ring().bytes().map(|b| b.unwrap()).collect();
    println!("bytes: {:?}", text(&bytes));

    // An empty deque.
    let mut q = VecDeque::new();
    let mut one = [b'.'];
    println!("empty read: {}", res(q.read(&mut one)));
    println!("empty read_exact 0: {}", res(q.read_exact(&mut [])));
    println!("empty read_exact 1: {} {:?}", res(q.read_exact(&mut one)), text(&one));
    let mut v = Vec::new();
    println!("empty read_to_end: {} {:?}", res(q.read_to_end(&mut v)), text(&v));
    let mut s = String::new();
    println!("empty read_to_string: {} {s:?}", res(q.read_to_string(&mut s)));
    println!("empty write none: {} rest {:?}", res(q.write(b"")), rest(&q));

    // `Write` appends at the back, past the wrap.
    let mut q = ring();
    println!("write: {}", res(q.write(b"gh")));
    println!("write_all: {}", res(q.write_all(b"ij")));
    println!("write!: {}", res(write!(q, "{}", 42)));
    println!("flush: {} rest {:?}", res(q.flush()), rest(&q));
    let mut s = String::new();
    println!("round trip: {} {s:?}", res(q.read_to_string(&mut s)));
    println!("END");
}
"#;
    let dir = package("vec-deque-io", &format!("{BODY}\n#[tide::main]\nfn main() {{ run(); }}\n"));
    checked(&mut driver(&dir));
    let tide = checked(&mut Command::new(dir.join("out/demo")));
    let rust_code = dir.join("rust.rs");
    std::fs::write(&rust_code, format!("{BODY}\nfn main() {{ run(); }}\n")).unwrap();
    plain_rustc(&rust_code, &dir.join("rust"));
    let rust = checked(&mut Command::new(dir.join("rust")));
    let transcript = String::from_utf8(rust.stdout).unwrap();
    assert!(transcript.ends_with("END\n"), "{transcript}");
    assert!(tide.stderr.is_empty(), "{}", stderr(&tide));
    assert_eq!(String::from_utf8(tide.stdout).unwrap(), transcript);
}

#[test]
fn executable_contracts_run_in_debug_test_and_erase_in_release() {
    for (name, source, count, marker) in [
        (
            "executable-contracts",
            include_str!("../../../conformance/cases/contracts/executable.rs"),
            16,
            "CONTRACTS_RAN\n",
        ),
        (
            "scoped-clauses",
            include_str!("../../../conformance/cases/contracts/scoped_clauses.rs"),
            5,
            "SCOPED_CLAUSES_RAN\n",
        ),
        (
            "clause-expressions",
            include_str!("../../../conformance/cases/contracts/clause_expressions.rs"),
            3,
            "CLAUSE_EXPRESSIONS_RAN\n",
        ),
    ] {
        let dir = package(name, source);
        for profile in ["debug", "release"] {
            for test in [false, true] {
                let mut build = driver(&dir);
                build.arg(format!("--tide-profile={profile}"));
                if test {
                    build.arg("--test");
                }
                checked(&mut build);
                let mut run = Command::new(dir.join(if test { "out/demo-test" } else { "out/demo" }));
                if test { run.args(["--skip", "__tide_contract_tests::"]); }
                let out = checked(&mut run);
                if test {
                    assert!(
                        String::from_utf8_lossy(&out.stdout).contains(&format!("{count} passed")),
                        "{}",
                        String::from_utf8_lossy(&out.stdout)
                    );
                } else {
                    assert_eq!(out.stdout, marker.as_bytes());
                }
            }
        }
    }
}

#[test]
fn contract_requires_body_keeps_direct_call_and_panic_locations() {
    for (name, attr, body, blamed_line, expected) in [
        ("nested", "", "f(y)", "f(y)", "requires violated: x > 0"),
        ("panic", "", "xs[i]", "xs[i]", "index out of bounds"),
        (
            "tracked-panic",
            "#[track_caller]",
            "xs[i]",
            "fn main() { g(0, &[], 0); }",
            "index out of bounds",
        ),
    ] {
        let source = format!(
            r#"
#[requires(x > 0)]
fn f(x: i32) -> i32 {{ x }}
#[requires(y >= 0)]
{attr}
fn g(y: i32, xs: &[i32], i: usize) -> i32 {{
    {body}
}}
#[tide::main]
fn main() {{ g(0, &[], 0); }}
"#
        );
        let dir = package(&format!("contract-body-{name}"), &source);
        let build = checked(&mut driver(&dir));
        assert!(
            !stderr(&build).contains("unnecessary braces"),
            "{}",
            stderr(&build)
        );
        let out = Command::new(dir.join("out/demo"))
            .stdin(Stdio::null())
            .output()
            .unwrap();
        assert_eq!(out.status.code(), Some(101));
        let line = source
            .lines()
            .position(|line| line.trim() == blamed_line)
            .unwrap()
            + 1;
        let message = stderr(&out);
        assert!(message.contains(expected), "{message}");
        assert!(message.contains(&format!("demo.rs:{line}:")), "{message}");
        if name == "nested" {
            assert!(message.contains(&format!("demo.rs:{line}:5")), "{message}");
        }
    }
}

#[test]
fn contract_requires_keeps_body_items_out_of_clause_scope() {
    for (name, qualifier, attr) in [("const", "const", ""), ("tracked", "", "#[track_caller]")] {
        for (local_limit, argument, exit) in [(1, 50, 0), (1000, 150, 101)] {
            let source = format!(
                r#"
#![deny(unused_braces)]
const LIMIT: i32 = 100;
#[requires(x < LIMIT)]
{attr}
{qualifier} fn value(x: i32) -> i32 {{
    const LIMIT: i32 = {local_limit};
    x + LIMIT - LIMIT
}}
#[tide::main]
fn main() {{ assert_eq!(value({argument}), {argument}); }}
"#
            );
            let dir = package(
                &format!("contract-item-scope-{name}-{local_limit}"),
                &source,
            );
            checked(driver(&dir).arg("--tide-profile=debug"));
            let out = Command::new(dir.join("out/demo"))
                .stdin(Stdio::null())
                .output()
                .unwrap();
            assert_eq!(out.status.code(), Some(exit), "{}", stderr(&out));
            if exit != 0 {
                assert!(
                    stderr(&out).contains("requires violated: x < LIMIT"),
                    "{}",
                    stderr(&out)
                );
            }
        }
    }
}

#[test]
fn contract_requires_never_and_const_bodies_do_not_warn() {
    let dir = package(
        "contract-requires-warnings",
        r#"
#![deny(unreachable_code, unused_braces)]
#[requires(code != 0)]
fn die(code: i32) -> ! { panic!("die {code}") }
#[requires(code != 0)]
#[ensures(true)]
fn die_with_ensures(code: i32) -> ! { panic!("die {code}") }
#[requires(x > 0)]
const fn identity(x: i32) -> i32 { x }
const VALUE: i32 = identity(1);
#[tide::main]
fn main() {
    assert_eq!(VALUE + identity(2), 3);
    let _ = die as fn(i32) -> !;
    let _ = die_with_ensures as fn(i32) -> !;
}
"#,
    );
    for profile in ["debug", "release"] {
        let out = checked(driver(&dir).arg(format!("--tide-profile={profile}")));
        assert!(!stderr(&out).contains("warning:"), "{}", stderr(&out));
        checked(&mut Command::new(dir.join("out/demo")));
    }
}

#[test]
fn contract_rejects_colliding_generated_operator_spans() {
    let dir = package("contract-generated-spans", "collision::clause!();");
    let generator = dir.join("generator.rs");
    std::fs::write(
        &generator,
        r##"
extern crate proc_macro;
#[proc_macro]
pub fn clause(_: proc_macro::TokenStream) -> proc_macro::TokenStream {
    "#[tide_macros::requires({ let p = true; p ==> true })] fn probe() {}".parse().unwrap()
}
"##,
    )
    .unwrap();
    let library = dir.join(format!(
        "{}collision{}",
        std::env::consts::DLL_PREFIX,
        std::env::consts::DLL_SUFFIX
    ));
    let rustc = std::env::var_os("RUSTC").unwrap_or("rustc".into());
    checked(
        Command::new(&rustc)
            .args([
                "--edition=2024",
                "--crate-name=collision",
                "--crate-type=proc-macro",
            ])
            .arg(&generator)
            .arg("-o")
            .arg(&library),
    );
    let macros = std::fs::read_dir(std_dir())
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .find(|p| {
            p.file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .starts_with("libtide_macros-")
        })
        .unwrap();
    let output = Command::new(&rustc)
        .args(["--edition=2024", "--crate-type=lib", "--extern"])
        .arg(format!("collision={}", library.display()))
        .arg("--extern")
        .arg(format!("tide_macros={}", macros.display()))
        .arg(dir.join("package/src/demo.rs"))
        .arg("--out-dir")
        .arg(dir.join("out"))
        .stdin(Stdio::null())
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    assert!(
        stderr(&output).contains("ambiguous clause spans"),
        "{}",
        stderr(&output)
    );
}

#[test]
fn contract_preserves_source_conjunct_and_caller_blame() {
    let source = r#"
struct Probe;
impl Drop for Probe {
    fn drop(&mut self) {
        if let Some(v) = std::__contract::take_violation() {
            println!("RECORD {:?} {} {} {} {}:{}:{}", v.clause.kind, v.clause.index,
                v.clause.conjunct, v.clause.function, v.file, v.line, v.column);
        }
    }
}

#[requires(
    a > 0 && "é".len() == 2 &&
    b   >  0
)]
fn pair(a: u32, b: u32) {}
#[tide::main]
fn main() {
    let _probe = Probe;
    pair(1, 0);
}
"#;
    let dir = package("contract-blame", source);
    checked(&mut driver(&dir));
    let out = Command::new(dir.join("out/demo"))
        .stdin(Stdio::null())
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(101));
    let message = stderr(&out);
    let line = source
        .lines()
        .position(|line| line.trim() == "pair(1, 0);")
        .unwrap()
        + 1;
    let blamed = format!("demo.rs:{line}:5");
    assert!(message.contains("requires violated: b   >  0"), "{message}");
    assert!(message.contains(&blamed), "{message}");
    let record = String::from_utf8_lossy(&out.stdout);
    assert!(record.contains("RECORD Requires 0 2 pair"), "{record}");
    assert!(record.contains(&blamed), "{record}");
}

#[test]
fn erased_contracts_still_resolve_type_and_borrow_check() {
    for (name, clause, expected) in [
        (
            "type",
            "#[ensures(result == true)] fn invalid() -> u32 { 1 }",
            "E0308",
        ),
        (
            "name",
            "#[ensures(result == missing)] fn invalid() -> u32 { 1 }",
            "E0425",
        ),
        (
            "move",
            "#[ensures(drop(result) == ())] fn invalid() -> Vec<u32> { vec![1] }",
            "E0382",
        ),
        (
            "nested-old",
            "#[ensures(old(old(n)) == result)] fn invalid(n: u32) -> u32 { n }",
            "nested old",
        ),
        (
            "bound-old",
            "#[ensures(forall |i: usize| i < 2 ==> old(i) == i)] fn invalid(i: usize) {}",
            "bound variable",
        ),
        (
            "shared-old",
            "struct Token(u32); #[ensures(old(t).0 == result)] fn invalid(t: &Token) -> u32 { t.0 }",
            "Clone",
        ),
        (
            "alias-mut",
            "type Mut<'a> = &'a mut u32; #[ensures(*x == 4)] fn invalid(x: Mut<'_>) { *x = 4; }",
            "mutable-reference alias in ensures is not supported",
        ),
        (
            "const-old",
            "#[ensures(result == old(n))] const fn invalid(n: u32) -> u32 { n }",
            "old snapshots on const fn are not supported",
        ),
        (
            "const-bad",
            "#[requires(n > 0)] const fn invalid(n: u32) -> u32 { n } const BAD: u32 = invalid(0);",
            "requires violated",
        ),
        (
            "pattern-old",
            "#[ensures(matches!(result, Some(n) if old(n) == n))] fn invalid(n: u8) -> Option<u8> { Some(n) }",
            "bound variable",
        ),
        (
            "const-method",
            "struct S; impl S { #[ensures(true)] const fn invalid(&self) {} }",
            "ensures on a const method is not supported",
        ),
        (
            "const-pattern",
            "#[ensures(result == n)] const fn invalid((n,): (u8,)) -> u8 { n }",
            "const ensures needs identifier parameters",
        ),
        (
            "const-entry",
            "struct S(u32); #[ensures(result == s.0)] const fn invalid(s: S) -> u32 { s.0 }",
            "const ensures needs Copy entry parameters",
        ),
        (
            "for-old",
            "#[ensures(matches!({ let mut s = 0; for n in 0..2 { s += old(n); } s }, 1))] fn invalid(n: u8) {}",
            "bound variable",
        ),
        (
            "while-old",
            "#[ensures(matches!({ let mut it = Some(1); let mut s = 0; while let Some(n) = it { s += old(n); it = None; } s }, 1))] fn invalid(n: u8) {}",
            "bound variable",
        ),
        (
            "format-unused-implication",
            "#[ensures(format!(\"{p}\", p ==> result) == \"true\")] fn invalid(p: bool) -> bool { false }",
            "argument never used",
        ),
        (
            "format-leading-space",
            "#[ensures(result == format!(\"{ n}\"))] fn invalid(n: u8) -> String { String::new() }",
            "invalid format string",
        ),
        (
            "format-raw-capture",
            "#[ensures(result == format!(\"{r#n}\"))] fn invalid(n: u8) -> String { String::new() }",
            "invalid format string",
        ),
    ] {
        let dir = package(
            &format!("contract-reject-{name}"),
            &format!("{clause}\n#[tide::main] fn main() {{}}"),
        );
        for profile in ["debug", "release"] {
            let out = driver(&dir)
                .arg(format!("--tide-profile={profile}"))
                .stdin(Stdio::null())
                .output()
                .unwrap();
            if name == "const-bad" && profile == "release" {
                assert!(out.status.success(), "{}", stderr(&out));
                continue;
            }
            assert_eq!(out.status.code(), Some(1), "{}", stderr(&out));
            assert!(stderr(&out).contains(expected), "{}", stderr(&out));
            if name == "alias-mut" {
                assert!(!stderr(&out).contains("E0500"), "{}", stderr(&out));
            }
            assert!(!dir.join("out/demo").exists());
        }
    }
}

#[test]
fn contract_const_sampled_test_has_an_explicit_limit() {
    let dir = package(
        "const-sampled-limit",
        "fn positive(n: u32) -> bool { n > 0 } #[requires(positive(n))] const fn sample(n: u32) -> u32 { n } #[invariants] const fn looped(mut n: u32) { while n > 0 { invariant!(positive(n)); n -= 1; } }",
    );
    let output = driver(&dir)
        .args(["--crate-type=lib", "--test"])
        .stdin(Stdio::null())
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    assert!(
        stderr(&output).contains("sampled contracts on const fn are not supported in test builds"),
        "{}",
        stderr(&output)
    );
    assert!(!stderr(&output).contains("E0015"), "{}", stderr(&output));
    checked(driver(&dir).args(["--crate-type=lib", "--test", "--tide-profile=release"]));
    checked(&mut Command::new(dir.join("out/demo-test")));
}

fn paired_test_driver(dir: &Path) -> Command {
    let mut command = driver(dir);
    command
        .args(["--crate-type=lib", "--test", "--tide-test-library"])
        .arg(dir.join("candidate/src/lib.rs"))
        .stdin(Stdio::null());
    command
}

#[test]
fn separate_library_tests_require_public_items_and_admit_both_crates() {
    let suite = "#[test] fn paints() { assert_eq!(tide_candidate::pigment(), 7); }";
    let dir = package("separate-library-tests", suite);
    let candidate = dir.join("candidate/src/lib.rs");
    std::fs::create_dir_all(candidate.parent().unwrap()).unwrap();
    let test = |emit: &str| {
        let mut command = paired_test_driver(&dir);
        command.arg(format!("--emit={emit}"));
        command
    };
    let executable = dir.join("out/demo-test");
    for emit in ["metadata", "link"] {
        let library = "fn pigment() -> u8 { 7 } #[cfg(test)] mod writer { compile_error!(\"writer tests enabled\"); }";
        std::fs::write(&candidate, format!("pub {library}")).unwrap();
        checked(&mut test(emit));
        if emit == "link" {
            let run = checked(Command::new(&executable).arg("--exact").arg("paints"));
            assert!(String::from_utf8_lossy(&run.stdout).contains("1 passed"));
        }
        std::fs::write(&candidate, format!("pub(crate) {library}")).unwrap();
        let out = test(emit).output().unwrap();
        assert_eq!(out.status.code(), Some(1), "{}", stderr(&out));
        assert!(stderr(&out).contains("E0603"), "{}", stderr(&out));
        assert!(std::fs::read_dir(dir.join("out")).unwrap().next().is_none());
    }

    let unsafe_library = "pub fn pigment() -> u8 { unsafe { 7 } }";
    let unsafe_suite = "#[test] fn paints() { unsafe { tide_candidate::pigment(); } }";
    for (library, suite) in [
        (unsafe_library, suite),
        ("pub fn pigment() -> u8 { 7 }", unsafe_suite),
    ] {
        std::fs::write(&candidate, library).unwrap();
        std::fs::write(dir.join("package/src/demo.rs"), suite).unwrap();
        let out = test("link").output().unwrap();
        assert_eq!(out.status.code(), Some(1), "{}", stderr(&out));
        assert!(stderr(&out).contains("TD0901"), "{}", stderr(&out));
        assert!(!executable.exists());
    }

    let out = test("link")
        .arg("--tide-dep-dir")
        .arg(dir.join("candidate"))
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2), "{}", stderr(&out));
    assert!(stderr(&out).contains("trusted dependencies must be separate"));
}

#[test]
fn separate_library_tests_run_the_library_sampled_clauses() {
    // A clause that calls the library's own method is sampled: the paired suite's run checks it.
    let suite = r#"
#[test]
#[should_panic(expected = "requires violated: !self.full()")]
fn second_fill() { let mut g = tide_candidate::Gauge::default(); g.fill(); g.fill(); }
#[test]
fn one_fill() { let mut g = tide_candidate::Gauge::default(); g.fill(); assert!(g.full()); }
"#;
    let dir = package("separate-library-sampled-clauses", suite);
    let candidate = dir.join("candidate/src/lib.rs");
    std::fs::create_dir_all(candidate.parent().unwrap()).unwrap();
    std::fs::write(
        &candidate,
        r#"
#[derive(Default)]
pub struct Gauge { level: u8 }
impl Gauge {
    pub fn full(&self) -> bool { self.level == 1 }
    #[requires(!self.full())]
    #[ensures(self.full())]
    pub fn fill(&mut self) { self.level += 1; }
}
"#,
    )
    .unwrap();
    checked(&mut paired_test_driver(&dir));
    let run = checked(&mut Command::new(dir.join("out/demo-test")));
    assert!(String::from_utf8_lossy(&run.stdout).contains("2 passed"));
}

#[test]
fn separate_library_tests_cannot_read_each_others_sources() {
    let dir = package("separate-library-source-reads", "");
    let candidate = dir.join("candidate/src/lib.rs");
    let suite = dir.join("package/src/demo.rs");
    std::fs::create_dir_all(candidate.parent().unwrap()).unwrap();
    for (reader, target, prefix) in [
        (&candidate, &suite, ""),
        (&suite, &candidate, "extern crate tide_candidate;"),
    ] {
        std::fs::write(target, "// foreign source canary").unwrap();
        std::fs::write(
            reader,
            format!("{prefix}\ncore::compile_error!(core::include_str!({target:?}));"),
        )
        .unwrap();
        let out = paired_test_driver(&dir).output().unwrap();
        let diagnostic = stderr(&out);
        assert_eq!(out.status.code(), Some(1), "{diagnostic}");
        assert!(diagnostic.contains("TD0520"), "{diagnostic}");
        assert!(
            !diagnostic.contains("foreign source canary"),
            "{diagnostic}"
        );
    }
}

#[test]
fn separate_library_tests_accept_local_impls_and_check_pure_at_use() {
    let suite = r#"
use tide_candidate as solution;
mod test {
    use crate::solution::Shape;

    fn sides<T: Shape>(shape: T) -> u8 { shape.sides() }
    fn empty() {}

    #[test] fn counts_sides() { assert_eq!(sides(empty as fn()), 3); }
}
"#;
    let dir = package("separate-library-impls-and-clauses", suite);
    let candidate = dir.join("candidate/src/lib.rs");
    std::fs::create_dir_all(candidate.parent().unwrap()).unwrap();
    std::fs::write(
        &candidate,
        r#"
pub trait Shape { fn sides(&self) -> u8; }
impl Shape for fn() {
    #[inline]
    fn sides(&self) -> u8 { 3 }
}
pub fn require_pure<T: Pure>() {}
"#,
    )
    .unwrap();
    checked(&mut paired_test_driver(&dir));
    let run =
        checked(Command::new(dir.join("out/demo-test")).args(["--exact", "test::counts_sides"]));
    assert!(String::from_utf8_lossy(&run.stdout).contains("1 passed"));

    std::fs::write(
        dir.join("package/src/demo.rs"),
        r#"
#[test]
fn check_clause() { tide_candidate::require_pure::<std::time::Clock>(); }
"#,
    )
    .unwrap();
    let out = paired_test_driver(&dir).output().unwrap();
    let diagnostic = stderr(&out);
    assert_eq!(out.status.code(), Some(1), "{diagnostic}");
    assert!(
        diagnostic
            .lines()
            .any(|line| line.contains("demo.rs:")
                && line.contains("TD0517: `Clock` is never `Pure`")),
        "{diagnostic}"
    );
    assert!(!dir.join("out/demo-test").exists());
}

#[test]
fn arc_and_weak_build_and_execute_against_production_std() {
    let source = format!(
        "{}\n{}",
        include_str!("../../../conformance/cases/authority/global_arc_accept.rs"),
        r#"
#[tide::main]
fn main(root: Root) {
    use std::hash::{Hash, Hasher};
    use std::sync::Weak;
    fn send_sync<T: Send + Sync>() {}
    send_sync::<Arc<u32>>();
    send_sync::<Weak<u32>>();
    {
        let (_arc, _weak);
        let value = String::from("borrowed payload");
        _arc = Arc::new(&value, &root.tasks);
        _weak = Arc::downgrade(&_arc, &root.tasks);
    }
    struct Value;
    let opaque = Arc::new(Value, &root.tasks);
    let cloned = opaque.clone();
    let weak = Arc::downgrade(&opaque, &root.tasks).clone();
    assert_eq!(format!("{:?}", weak), "(Weak)");
    drop(opaque);
    assert!(weak.upgrade().is_some());
    drop(cloned);
    assert!(weak.upgrade().is_none());
    assert_eq!(edit(&root.tasks), Some(3));
    let mut a = Arc::new(String::from("old"), &root.tasks);
    let b = a.clone();
    Arc::make_mut(&mut a).push_str(" new");
    assert_eq!(&*a, "old new");
    assert_eq!(&*b, "old");
    let weak: Weak<String> = Arc::downgrade(&a, &root.tasks);
    let other = weak.clone();
    assert_eq!(weak.upgrade().unwrap(), a);
    // With only weak observers, make_mut dissociates them without cloning the value.
    Arc::make_mut(&mut a).push('!');
    assert!(weak.upgrade().is_none());
    drop(a);
    assert!(other.upgrade().is_none());
    let weak = Arc::downgrade(&b, &root.tasks);
    assert!(weak.upgrade().is_some());
    drop(b);
    assert!(weak.upgrade().is_none());
    let a = Arc::new(3u32, &root.tasks);
    let b = Arc::new(4u32, &root.tasks);
    assert!(a < b && a <= b && b > a && b >= a && a != b);
    assert_eq!(a.cmp(&b), std::cmp::Ordering::Less);
    assert_eq!(format!("{} {:?}", a, a), "3 3");
    assert_eq!(format!("{:?}", Arc::downgrade(&a, &root.tasks)), "(Weak)");
    let mut wrapped = std::hash::DefaultHasher::new();
    let mut plain = std::hash::DefaultHasher::new();
    a.hash(&mut wrapped); 3u32.hash(&mut plain);
    assert_eq!(wrapped.finish(), plain.finish());
    let transferred = a.clone();
    root.tasks.scope::<_, std::task::TaskError, _>(|s| {
        assert_eq!(s.spawn(move || Ok(*transferred)).join().unwrap(), 3);
        Ok(())
    }).unwrap();
}
"#,
    );
    let dir = package("arc-weak", &source);
    for profile in ["debug", "release"] {
        let build = checked(driver(&dir).arg(format!("--tide-profile={profile}")));
        assert!(!stderr(&build).contains("TD0520"), "{}", stderr(&build));
        checked(&mut Command::new(dir.join("out/demo")));
    }
}

#[test]
fn arc_and_weak_rejections_use_production_std() {
    for (name, source, code) in [
        ("count", include_str!("../../../conformance/cases/authority/global_arc_count.rs").to_owned(), "E0599"),
        ("payload", include_str!("../../../conformance/cases/authority/global_arc_payload.rs").to_owned(), "TD0606"),
        ("weak-pure", "fn pure(_: impl Pure) {} fn f(a: std::sync::Weak<u32>) { pure(a); }".into(), "TD0517"),
        ("new-tasks", "fn f() { let _ = std::sync::Arc::new(1u32); }".into(), "TD0505"),
        ("downgrade-tasks", "fn f(a: &std::sync::Arc<u32>) { let _ = std::sync::Arc::downgrade(a); }".into(), "TD0505"),
        ("arc-send", "fn send<T: Send>() {} fn f() { send::<std::sync::Arc<std::rc::Rc<u32>>>(); }".into(), "E0277"),
        ("weak-sync", "fn sync<T: Sync>() {} fn f() { sync::<std::sync::Weak<std::cell::Cell<u32>>>(); }".into(), "E0277"),
        ("pointer", "fn f(a: std::sync::Arc<u32>) { let _ = format!(\"{:p}\", a); }".into(), "TD0520"),
    ].into_iter().chain(["weak_count", "get_mut", "try_unwrap", "into_inner"].map(|method| (
        method,
        format!("fn f(mut a: std::sync::Arc<u32>) {{ let _ = std::sync::Arc::{method}(&mut a); }}"),
        "E0599",
    ))) {
        let dir = package(&format!("arc-reject-{name}"), &source);
        let out = driver(&dir).args(["--crate-type=lib", "--emit=metadata"])
            .stdin(Stdio::null()).output().unwrap();
        assert_eq!(out.status.code(), Some(1), "{name}: {}", stderr(&out));
        assert!(stderr(&out).contains(code), "{name}: {}", stderr(&out));
    }
}

#[test]
fn dbg_builds_and_executes_against_production_std() {
    let source = format!(
        "{}\n{}",
        include_str!("../../../conformance/cases/authority/sink_dbg_debug.rs"),
        r#"
#[tide::main]
fn main() {
    assert_eq!(half(8), 4);
    let __tide_dbg_value = String::from("moved");
    assert_eq!(dbg!(__tide_dbg_value,), "moved");
    let mut n = 0;
    assert_eq!(dbg!({ n += 1; n }, { n += 1; n },), (1, 2));
    assert_eq!(n, 2);
    assert_eq!(dbg!((1, 2)), (1, 2));
    dbg!();
    #[derive(Debug)]
    struct Value { n: u32 }
    assert_eq!(dbg!(Value { n: 3 }).n, 3);
    let cell = std::cell::RefCell::new(3);
    if let Some(v) = dbg!(Some(cell.borrow())) { assert_eq!(*v, 3); }
    *cell.borrow_mut() = 4;
}
#[test]
fn simulated_dbg() {
    let sim = Root::sim(0);
    assert_eq!(dbg!(7), 7);
    assert_eq!(std::par::join(|| dbg!(1), || dbg!(2)), (1, 2));
    assert_eq!(dbg!((1, 2)), (1, 2));
    let records = sim.records();
    assert_eq!(records.len(), 7);
    assert!(records.iter().all(|r| r.sink == std::sim::Sink::Dbg));
    assert!(records[0].message.ends_with("7 = 7"));
    assert!(records[1].message.ends_with("1 = 1"));
    assert!(records[2].message.ends_with("2 = 2"));
    assert_eq!(records[4].message, "    1,");
    assert_eq!(records[5].message, "    2,");
    assert_eq!(records[6].message, ")");
}
"#
    );
    let dir = package("dbg", &source);
    let build = checked(&mut driver(&dir));
    assert!(!stderr(&build).contains("TD0520"), "{}", stderr(&build));
    let out = checked(&mut Command::new(dir.join("out/demo")));
    assert!(out.stdout.is_empty());
    let lines: Vec<_> = stderr(&out).lines().map(str::to_owned).collect();
    assert!(lines[0].ends_with("/demo.rs:4:5] x = 8"), "{lines:?}");
    assert!(
        lines[1].ends_with("__tide_dbg_value = \"moved\""),
        "{lines:?}"
    );
    assert!(
        lines[2].ends_with("= 1") && lines[3].ends_with("= 2"),
        "{lines:?}"
    );
    assert_eq!(&lines[5..8], ["    1,", "    2,", ")"]);
    assert!(
        lines[8].starts_with('[') && lines[8].ends_with(']'),
        "{lines:?}"
    );
    assert!(
        lines[9].contains("] Value {") && lines[9].ends_with(" = Value {"),
        "{lines:?}"
    );
    assert_eq!(&lines[10..12], ["    n: 3,", "}"]);
    checked(driver(&dir).args(["--crate-type=lib", "--test"]));
    checked(&mut Command::new(dir.join("out/demo-test")));

    let dir = package(
        "dbg-release",
        include_str!("../../../conformance/cases/authority/sink_dbg_release.rs"),
    );
    let out = driver(&dir)
        .args([
            "--crate-type=lib",
            "--emit=metadata",
            "--tide-profile=release",
        ])
        .stdin(Stdio::null())
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1), "{}", stderr(&out));
    assert!(stderr(&out).contains("TD0507"), "{}", stderr(&out));
    assert!(!stderr(&out).contains("TD0520"), "{}", stderr(&out));
}

#[test]
fn arc_weak_simulation_uses_runtime_context() {
    let dir = package(
        "arc-simulation",
        r#"
use std::sync::Arc;

struct Notify(std::sync::mpsc::Sender<()>);
impl Drop for Notify {
    fn drop(&mut self) { self.0.send(()).unwrap(); }
}

#[cfg(test)]
fn poll_trace(seed: u64) -> String {
    let sim = Root::sim(seed);
    let (notify, notification) = std::sync::mpsc::channel(&sim.tasks);
    let a = Arc::new(Notify(notify), &sim.tasks);
    let weak = Arc::downgrade(&a, &sim.tasks);
    sim.tasks.scope::<_, std::task::TaskError, _>(|s| {
        let (tx, rx) = std::sync::mpsc::channel::<()>(&sim.tasks);
        let child = s.spawn(move || {
            assert!(rx.recv().is_err());
            drop(a);
            Ok(())
        });
        // Drop wakes the receiver without a scheduling point: the dropper is now runnable.
        drop(tx);
        let mut polls = 0;
        while weak.upgrade().is_some() {
            polls += 1;
            assert!(polls < 1000, "weak polling starved the runnable dropper");
        }
        // The payload destructor re-enters the runtime; no Arc runtime lock may be held.
        notification.recv().unwrap();
        child.join().unwrap();
        Ok(())
    }).unwrap();
    drop(weak);
    let trace = sim.trace();
    for op in ["arc.new", "arc.downgrade", "weak.upgrade"] {
        assert!(trace.contains(op), "missing Arc operation");
    }
    trace
}

#[test]
fn polling_and_replay() {
    for seed in 0..8 {
        assert_eq!(poll_trace(seed), poll_trace(seed));
    }
    // Emit outside the simulation so the host can compare two fresh processes.
    print!("{}", poll_trace(17));
}

#[test]
fn completed_simulation_results_can_be_read_and_compared() {
    #[cfg(test)]
    fn run(seed: u64) -> Arc<Vec<u32>> {
        let sim = Root::sim(seed);
        Arc::new(vec![1, 2, 3], &sim.tasks)
    }
    let a = run(1);
    let b = run(2);
    assert_eq!(a.len(), 3);
    assert_eq!(a, b);
    assert_eq!(a.clone(), b);
}

#[test]
fn transient_upgrades_allow_both_cow_outcomes() {
    struct Value(u32);
    impl Clone for Value {
        fn clone(&self) -> Self { Self(self.0 + 1) }
    }
    let mut counts = [0usize; 2];
    for seed in 0..256 {
        let sim = Root::sim(seed);
        let mut a = Arc::new(Value(0), &sim.tasks);
        let weak = Arc::downgrade(&a, &sim.tasks);
        let value = sim.tasks.scope::<_, std::task::TaskError, _>(|s| {
            let child = s.spawn(move || {
                // No intervening operation keeps this temporary alive for the scheduler.
                drop(weak.upgrade());
                Ok(())
            });
            let value = Arc::make_mut(&mut a).0;
            child.join().unwrap();
            Ok(value)
        }).unwrap();
        match value {
            0 => counts[0] += 1,
            1 => counts[1] += 1,
            _ => panic!("unexpected COW value"),
        }
    }
    assert!(counts[0] > 0 && counts[1] > 0, "transient upgrade outcomes: {:?}", counts);
}

#[test]
fn lifecycle_and_copy_on_write() {
    let sim = Root::sim(0);
    let before = sim.trace();
    let mut a = Arc::new(1, &sim.tasks);
    assert_ne!(sim.trace(), before);
    let before = sim.trace();
    let original = a.clone();
    assert_eq!(sim.trace(), before);
    let weak = Arc::downgrade(&a, &sim.tasks);
    assert_ne!(sim.trace(), before);
    let before = sim.trace();
    let other = weak.clone();
    assert_eq!(sim.trace(), before);
    *Arc::make_mut(&mut a) = 2;
    assert!(sim.trace().contains("arc.make_mut"));
    assert_eq!(*a, 2);
    assert_eq!(*weak.upgrade().unwrap(), 1);
    drop(original);
    assert!(other.upgrade().is_none());
    let before = sim.trace().len();
    drop((a, weak, other));
    assert!(!sim.trace()[before..].contains("\"wait\":\"ready\""));
}

#[test]
#[should_panic(expected = "simulation handle used outside its simulation")]
fn ended_simulation_upgrade() {
    let weak;
    {
        let sim = Root::sim(0);
        weak = Arc::downgrade(&Arc::new(1, &sim.tasks), &sim.tasks);
    }
    weak.upgrade();
}

#[test]
#[should_panic(expected = "simulation handle used outside its simulation")]
fn foreign_simulation_downgrade() {
    let a;
    {
        let sim = Root::sim(0);
        a = Arc::new(1, &sim.tasks);
    }
    let sim = Root::sim(1);
    Arc::downgrade(&a, &sim.tasks);
}

#[test]
fn outliving_handles_drop_in_another_simulation() {
    let (a, weak);
    {
        let sim = Root::sim(0);
        a = Arc::new(1, &sim.tasks);
        weak = Arc::downgrade(&a, &sim.tasks);
    }
    let sim = Root::sim(1);
    let before = sim.trace();
    drop(a);
    drop(weak);
    assert_eq!(sim.trace(), before);
}
"#,
    );
    checked(driver(&dir).args(["--crate-type=lib", "--test"]));
    let executable = dir.join("out/demo-test");
    let out = Command::new("timeout")
        .arg("30")
        .arg(&executable)
        .arg("--test-threads=4")
        .stdin(Stdio::null())
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&out.stdout),
        stderr(&out)
    );
    assert!(
        String::from_utf8_lossy(&out.stdout).contains("7 passed"),
        "{}",
        String::from_utf8_lossy(&out.stdout)
    );
    let replay = || {
        let out = checked(Command::new("timeout").arg("30").arg(&executable).args([
            "--exact",
            "polling_and_replay",
            "--nocapture",
        ]));
        String::from_utf8_lossy(&out.stdout)
            .lines()
            .filter_map(|line| line.find('{').map(|start| line[start..].to_owned()))
            .collect::<Vec<_>>()
    };
    let first = replay();
    assert!(first.iter().any(|line| line.contains("weak.upgrade")));
    let second = replay();
    assert_eq!(first, second);
}

#[test]
fn task_lifecycle_builds_and_runs_against_production_std() {
    let source = include_str!("../../../conformance/cases/tasks/task_lifecycle.rs");
    let dir = package("task-lifecycle", source);
    for profile in ["debug", "release"] {
        checked(driver(&dir).arg(format!("--tide-profile={profile}")));
        let out = checked(&mut Command::new(dir.join("out/demo")));
        assert_eq!(out.stdout, b"TASKS_RAN\n");
        assert!(out.stderr.is_empty(), "{}", stderr(&out));
    }
}

#[test]
fn shared_counter_runs_against_production_std() {
    let source = include_str!("../../../conformance/cases/tasks/shared_counter.rs");
    let dir = package("shared-counter", source);
    checked(&mut driver(&dir));
    checked(&mut Command::new(dir.join("out/demo")));
}

#[test]
fn shared_sets_execute_on_threads_and_under_simulation() {
    let dir = package(
        "shared-sets",
        include_str!("../../../conformance/cases/tasks/shared_sets.rs"),
    );
    checked(&mut driver(&dir));
    checked(Command::new("timeout").arg("60").arg(dir.join("out/demo")));
    checked(driver(&dir).args(["--crate-type=lib", "--test"]));
    let out = checked(Command::new("timeout").arg("120").arg(dir.join("out/demo-test")));
    assert!(
        String::from_utf8_lossy(&out.stdout).contains("4 passed"),
        "{}",
        String::from_utf8_lossy(&out.stdout)
    );
}

#[test]
fn parallel_regions_execute_on_both_runtimes_in_debug_and_release() {
    let dir = package(
        "parallel-regions",
        include_str!("../../../conformance/cases/tasks/parallel_regions.rs"),
    );
    for profile in ["debug", "release"] {
        checked(driver(&dir).arg(format!("--tide-profile={profile}")));
        let out = checked(Command::new("timeout").arg("30").arg(dir.join("out/demo")));
        assert_eq!(
            out.stdout,
            b"left\nright\nmap 0\nmap 1\nmap 2\nmap 3\nfor 0\nfor 3\nfor 6\nfor 9\n"
        );
        assert!(out.stderr.is_empty(), "{}", stderr(&out));
        checked(driver(&dir).args([
            "--crate-type=lib",
            "--test",
            &format!("--tide-profile={profile}"),
        ]));
        let out = checked(
            Command::new("timeout")
                .arg("60")
                .arg(dir.join("out/demo-test")),
        );
        assert!(
            String::from_utf8_lossy(&out.stdout).contains("4 passed"),
            "{}",
            String::from_utf8_lossy(&out.stdout)
        );
    }
}

#[test]
fn parallel_panics_keep_only_the_sequential_sink_prefix() {
    for (name, body, expected) in [
        (
            "join",
            "std::par::join(|| { println!(\"left\"); panic!(\"winner\") }, || { println!(\"right\"); panic!(\"loser\") });",
            "left\n",
        ),
        (
            "map",
            "let _ = [0, 1, 2, 3].par_map(|i| { println!(\"{i}\"); if *i == 1 { panic!(\"winner\"); } *i });",
            "0\n1\n",
        ),
        (
            "for",
            "[0, 1, 2, 3].par_for(|i| { println!(\"{i}\"); if *i == 1 { panic!(\"winner\"); } });",
            "0\n1\n",
        ),
    ] {
        for simulated in [false, true] {
            let source = if simulated {
                format!(
                    r#"#[test] fn run() {{
                    let sim = Root::sim(7);
                    let panic = sim.tasks.spawn_isolated(|| {{ {body} }}).unwrap_err();
                    assert!(panic.message.contains("winner"));
                    let output: String = sim.records().iter().map(|r| format!("{{}}\n", r.message)).collect();
                    assert_eq!(output, {expected:?});
                }}"#
                )
            } else {
                format!("#[tide::main] fn main() {{ {body} }}")
            };
            let dir = package(&format!("parallel-panic-{name}-{simulated}"), &source);
            for profile in ["debug", "release"] {
                let mut build = driver(&dir);
                build.arg(format!("--tide-profile={profile}"));
                if simulated {
                    build.args(["--crate-type=lib", "--test"]);
                }
                checked(&mut build);
                let out = Command::new("timeout")
                    .arg("30")
                    .arg(dir.join(if simulated {
                        "out/demo-test"
                    } else {
                        "out/demo"
                    }))
                    .stdin(Stdio::null())
                    .output()
                    .unwrap();
                assert_eq!(
                    out.status.code(),
                    Some(if simulated { 0 } else { 101 }),
                    "{}",
                    stderr(&out)
                );
                if simulated {
                    assert!(
                        String::from_utf8_lossy(&out.stdout).contains("1 passed"),
                        "{}",
                        stderr(&out)
                    );
                } else {
                    assert_eq!(out.stdout, expected.as_bytes());
                    assert!(stderr(&out).contains("winner"), "{}", stderr(&out));
                }
            }
        }
    }
}

#[test]
fn parallel_pure_rejections_use_the_existing_owner() {
    for (name, source, code) in [
        (
            "join",
            "pub fn run(clock: std::time::Clock) { std::par::join(move || clock.now(), || 1); }",
            "TD0610",
        ),
        (
            "capture",
            "pub fn run(clock: &std::time::Clock) { [1].par_map(|_| clock.now()); }",
            "TD0611",
        ),
        (
            "for",
            "pub fn run(clock: std::time::Clock) { [1].par_for(move |_| { clock.now(); }); }",
            "TD0610",
        ),
        (
            "elements",
            "pub fn run(xs: &[std::time::Clock]) { xs.par_map(|_| 1); }",
            "TD0612",
        ),
        (
            "sort",
            include_str!("../../../conformance/cases/tasks/par_sort_reject.rs"),
            "TD0612",
        ),
    ] {
        let dir = package(&format!("parallel-reject-{name}"), source);
        let out = driver(&dir)
            .args(["--crate-type=lib", "--emit=metadata"])
            .stdin(Stdio::null())
            .output()
            .unwrap();
        assert_eq!(out.status.code(), Some(1), "{}", stderr(&out));
        assert!(stderr(&out).contains(code), "{}", stderr(&out));
    }
}

#[test]
fn collections_execute_with_fixed_and_capability_seeded_keys() {
    let dir = package(
        "collections",
        include_str!("../../../conformance/cases/authority/collections_executable.rs"),
    );
    checked(&mut driver(&dir));
    let first = checked(&mut Command::new(dir.join("out/demo")));
    let second = checked(&mut Command::new(dir.join("out/demo")));
    assert!(!first.stdout.is_empty());
    assert_eq!(
        first.stdout, second.stdout,
        "fixed-key iteration changed across processes"
    );
    assert_ne!(
        first.stderr, second.stderr,
        "OS-seeded hash fingerprints repeated"
    );
}

#[test]
fn collections_disjoint_lookups_enforce_safety() {
    let dir = package(
        "collection-unchecked-lookup",
        r#"
#[tide::main]
fn main() {
    let mut map = std::collections::HashMap::from([(1, 10), (2, 20)]);
    // SAFETY: distinct keys satisfy the unchecked lookup's safety contract.
    let [first, missing, second] = unsafe { map.get_disjoint_unchecked_mut([&1, &0, &2]) };
    *first.unwrap() += 1;
    assert!(missing.is_none());
    *second.unwrap() += 2;
    assert_eq!(map[&1], 11);
    assert_eq!(map[&2], 22);
}
"#,
    );
    let out = driver(&dir).stdin(Stdio::null()).output().unwrap();
    assert_eq!(out.status.code(), Some(1), "{}", stderr(&out));
    assert!(stderr(&out).contains("TD0901"), "{}", stderr(&out));
    assert!(!dir.join("out/demo").exists());
    checked(driver(&dir).arg("--tide-grant=unsafe"));
    checked(&mut Command::new(dir.join("out/demo")));

    let dir = package(
        "collection-overlapping-lookup",
        r#"
#[tide::main]
fn main() {
    let mut map = std::collections::HashMap::from([(1, 10)]);
    let _ = map.get_disjoint_mut([&1, &1]);
}
"#,
    );
    checked(&mut driver(&dir));
    let out = Command::new(dir.join("out/demo"))
        .stdin(Stdio::null())
        .output()
        .unwrap();
    assert!(!out.status.success());
    assert!(
        stderr(&out).contains("duplicate keys found"),
        "{}",
        stderr(&out)
    );
}

#[test]
fn collections_cannot_draw_ambient_keys_through_a_generic_hasher() {
    for (name, expression) in [
        (
            "map",
            "std::collections::HashMap::<u8, u8, std::hash::RandomState>::default()",
        ),
        (
            "set",
            "std::collections::HashSet::<u8, std::hash::RandomState>::default()",
        ),
    ] {
        let dir = package(
            &format!("ambient-collection-{name}"),
            &format!("#[tide::main] fn main() {{ let _ = {expression}; }}"),
        );
        let out = driver(&dir).stdin(Stdio::null()).output().unwrap();
        assert_eq!(out.status.code(), Some(1), "{}", stderr(&out));
        assert!(
            stderr(&out).contains(
                "TD0520: `impl core::default::Default for std::hash::random::RandomState`"
            ),
            "{}",
            stderr(&out)
        );
        assert!(!dir.join("out/demo").exists());
    }
}

#[test]
fn task_failures_and_cancellation_build_and_run_against_production_std() {
    let source = include_str!("../../../conformance/cases/tasks/task_cancellation.rs");
    let dir = package("task-cancellation", source);
    for profile in ["debug", "release"] {
        checked(driver(&dir).arg(format!("--tide-profile={profile}")));
        let out = checked(&mut Command::new(dir.join("out/demo")));
        assert_eq!(out.stdout, b"CANCELLATION_RAN\n");
        assert!(out.stderr.is_empty(), "{}", stderr(&out));
    }
}

/// Limits, deadlines, shielding and isolation execute through the admitted std on both runtimes.
#[test]
fn task_controls_build_and_run_against_production_std() {
    let dir = package(
        "task-controls",
        include_str!("../../../conformance/cases/tasks/task_controls.rs"),
    );
    for profile in ["debug", "release"] {
        checked(driver(&dir).arg(format!("--tide-profile={profile}")));
        let out = checked(&mut Command::new(dir.join("out/demo")));
        assert_eq!(out.stdout, b"TASK_CONTROLS_RAN\n");
        assert!(
            stderr(&out).contains("contained task panic"),
            "{}",
            stderr(&out)
        );
        checked(driver(&dir).args(["--test", &format!("--tide-profile={profile}")]));
        let out = checked(
            Command::new(dir.join("out/demo-test")).args(["--exact", "simulated_controls"]),
        );
        assert!(String::from_utf8_lossy(&out.stdout).contains("1 passed"));
    }
}

/// Channels execute through the admitted std on the real and simulated runtimes.
#[test]
fn channels_build_and_run_against_production_std() {
    let dir = package(
        "channels",
        include_str!("../../../conformance/cases/tasks/channels.rs"),
    );
    for profile in ["debug", "release"] {
        checked(driver(&dir).arg(format!("--tide-profile={profile}")));
        let out = checked(&mut Command::new(dir.join("out/demo")));
        assert_eq!(out.stdout, b"CHANNELS_RAN\n");
        assert!(out.stderr.is_empty(), "{}", stderr(&out));
        checked(driver(&dir).args(["--test", &format!("--tide-profile={profile}")]));
        let out = checked(
            Command::new(dir.join("out/demo-test")).args(["--exact", "simulated_channels"]),
        );
        assert!(String::from_utf8_lossy(&out.stdout).contains("1 passed"));
    }
}

#[test]
fn admitted_channel_deadlock_lists_senders_receivers_and_call_sites() {
    let dir = package(
        "channel-deadlock",
        include_str!("../../../conformance/cases/tasks/channel_deadlock.rs"),
    );
    for profile in ["debug", "release"] {
        checked(
            driver(&dir)
                .args(["--crate-type=lib", "--test"])
                .arg(format!("--tide-profile={profile}")),
        );
        let out = Command::new(dir.join("out/demo-test"))
            .args(["--exact", "deadlock", "--nocapture"])
            .stdin(Stdio::null())
            .output()
            .unwrap();
        assert!(!out.status.success());
        let report = stderr(&out);
        assert!(
            report.contains("task 0 blocked in send(chan 1) at "),
            "{report}"
        );
        assert!(
            report.contains("task 1 blocked in recv(chan 0) at "),
            "{report}"
        );
        let last = report.lines().last().unwrap();
        assert!(last.contains("\"operation\":\"deadlock\""), "{report}");
        assert!(
            last.contains("\"blocked\":[{\"task\":0,\"operation\":\"send\",\"target\":1"),
            "{report}"
        );
        assert!(
            last.contains("{\"task\":1,\"operation\":\"recv\",\"target\":0"),
            "{report}"
        );
        assert!(last.contains("demo.rs"), "{report}");
    }
}

#[test]
fn admitted_limit_deadlock_lists_every_waiter_and_call_site() {
    let dir = package(
        "limit-deadlock",
        include_str!("../../../conformance/cases/tasks/limit_deadlock.rs"),
    );
    checked(driver(&dir).args(["--crate-type=lib", "--test"]));
    let out = Command::new(dir.join("out/demo-test"))
        .args(["--exact", "deadlock", "--nocapture"])
        .stdin(Stdio::null())
        .output()
        .unwrap();
    assert!(!out.status.success());
    let report = stderr(&out);
    for task in 0..2 {
        assert!(
            report.contains(&format!("task {task} blocked in limit(0) at ")),
            "{report}"
        );
    }
    let last = report.lines().last().unwrap();
    assert!(last.contains("\"operation\":\"deadlock\""), "{report}");
    assert!(
        last.contains("\"blocked\":[{\"task\":0,\"operation\":\"limit\""),
        "{report}"
    );
    assert!(
        last.contains("{\"task\":1,\"operation\":\"limit\""),
        "{report}"
    );
    assert!(last.contains("demo.rs"), "{report}");
}

/// `Clock`, `Instant`, `SystemTime` and `Duration` (bead fx-o8j) pass the driver and run: clock
/// readings and their arithmetic, real sleeps of at least their duration, `Cancelled` from a
/// sleep in a cancelled scope, and the panic of `system_time()` on a `monotonic_only()` clock.
#[test]
fn clock_and_time_build_and_run_against_production_std() {
    let source = include_str!("../../../conformance/cases/authority/clock_time.rs");
    let dir = package("clock-time", source);
    for profile in ["debug", "release"] {
        checked(driver(&dir).arg(format!("--tide-profile={profile}")));
        let out = checked(&mut Command::new(dir.join("out/demo")));
        assert_eq!(out.stdout, b"TIME_RAN\n");
        assert!(out.stderr.is_empty(), "{}", stderr(&out));
    }

    let source = "#[tide::main]\nfn main(root: Root) {\n    \
                  let clock = root.clock.monotonic_only();\n    \
                  println!(\"{}\", clock.now() <= clock.now());\n    \
                  let _ = clock.system_time();\n}\n";
    let dir = package("clock-monotonic-only", source);
    checked(&mut driver(&dir));
    let out = Command::new(dir.join("out/demo"))
        .stdin(Stdio::null())
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(101), "{}", stderr(&out));
    assert_eq!(out.stdout, b"true\n");
    assert!(
        stderr(&out).contains(
            "`system_time()` called on a `Clock` from `monotonic_only()`, which cannot read wall \
             time"
        ),
        "{}",
        stderr(&out)
    );
}

#[test]
fn checked_hello_builds_without_running_and_ignores_candidate_configuration() {
    let dir = package("hello", HELLO);
    std::fs::create_dir(dir.join("package/.cargo")).unwrap();
    std::fs::write(dir.join("package/.cargo/config.toml"), "[build]\nrustflags=['--cfg=attacker']\nrustc-wrapper='missing'\n[target.x86_64-unknown-linux-gnu]\nlinker='missing'\n").unwrap();
    std::fs::write(
        dir.join("package/rust-toolchain.toml"),
        "[toolchain]\nchannel='nonexistent'\n",
    )
    .unwrap();
    for _ in 0..2 {
        let out = checked(
            driver(&dir)
                .current_dir(dir.join("package"))
                .env("PATH", "/nonexistent")
                .env("TMPDIR", "/nonexistent")
                .env("RUSTFLAGS", "-Coverflow-checks=off -Clinker=missing")
                .env("RUSTC_WRAPPER", "missing")
                .env("RUSTUP_TOOLCHAIN", "missing")
                .env("LD_LIBRARY_PATH", "/nonexistent"),
        );
        assert!(out.stdout.is_empty(), "the driver executed the program");
        assert!(out.stderr.is_empty(), "{}", stderr(&out));
        assert!(dir.join("out/demo.interface.json").is_file());
        let out = checked(&mut Command::new(dir.join("out/demo")));
        assert_eq!(out.stdout, b"HELLO_RAN\n");
        std::fs::remove_file(dir.join("out/demo")).unwrap();
    }
}

#[test]
fn the_backstop_rejects_address_comparisons_in_every_profile() {
    let source = r#"
fn noop() {}
fn other() {}
trait Addr { fn f(&self, which: bool) -> impl PartialOrd; }
impl Addr for u8 {
    fn f(&self, which: bool) -> impl PartialOrd {
        if which { noop as fn() } else { other as fn() }
    }
}
#[tide::main]
fn main() {
    let first = "1".parse::<u8>().unwrap() == 1;
    println!("{}", 1u8.f(first) < 1u8.f(!first));
}
"#;
    let dir = package("profile-backstop", source);
    for emit in ["link", "metadata"] {
        for profile in ["debug", "release"] {
            let out = driver(&dir)
                .arg("--tide-test-backstop")
                .arg(format!("--emit={emit}"))
                .arg(format!("--tide-profile={profile}"))
                .output()
                .unwrap();
            assert_eq!(
                out.status.code(),
                Some(1),
                "{emit}/{profile}: {}",
                stderr(&out)
            );
            assert!(
                stderr(&out).contains("TD0520: `impl core::cmp::PartialOrd for fn()`"),
                "{emit}/{profile}: {}",
                stderr(&out)
            );
            assert!(out.stdout.is_empty());
            assert!(!dir.join("out/demo").exists());
            assert!(!dir.join("out/demo.interface.json").exists());
            assert_eq!(
                std::fs::read_dir(dir.join("out/.tide-cache"))
                    .unwrap()
                    .count(),
                0
            );
        }
    }
}

#[test]
fn executable_tests_use_the_production_prelude_and_do_not_run_main() {
    let dir = package("test", HELLO);
    let out = checked(driver(&dir).args(["--crate-type=lib", "--test"]));
    assert!(out.stdout.is_empty(), "the driver executed the test");
    let out = checked(Command::new(dir.join("out/demo-test")).arg("--nocapture"));
    assert!(String::from_utf8_lossy(&out.stdout).contains("1 passed"));
    assert!(String::from_utf8_lossy(&out.stdout).contains("TEST_RAN"));
    assert!(!String::from_utf8_lossy(&out.stdout).contains("HELLO_RAN"));
}

#[test]
fn property_test_harness_generates_shrinks_and_replays() {
    let dir = package(
        "property",
        r#"
use std::test::{Arbitrary, Gen};

#[allow(non_upper_case_globals)]
const error: u8 = 0;

#[test]
fn ordinary() {}

#[test]
fn sum(a: u8, b: u8) {
    assert!(a as u16 + b as u16 <= 300);
}

#[test]
fn vector(xs: Vec<i64>) {
    assert!(xs.iter().map(|x| *x as i128).sum::<i128>() < 100);
}

#[derive(Debug)]
struct Even(u8);
impl Arbitrary for Even {
    fn arbitrary(g: &mut Gen) -> Self { Self(u8::arbitrary(g) & !1) }
}

#[test]
fn even(x: Even) {
    assert!(x.0 < 10);
}

#[test]
fn same_failure(x: u8) {
    if x == 0 { panic!("zero is a different failure"); }
    assert!(x < 10);
}

#[derive(Debug)]
struct Positive(u8);
impl Arbitrary for Positive {
    fn arbitrary(g: &mut Gen) -> Self {
        loop {
            let n = u8::arbitrary(g);
            if n != 0 { return Self(n); }
        }
    }
}
#[test]
fn positive(p: Positive) { assert_eq!(p.0, 0); }

#[test]
fn returned_error(x: u8) -> Result<(), String> {
    if x > 3 { Err(String::from("limit")) } else { Ok(()) }
}

type NestedResult = Result<Result<(), String>, String>;
#[test]
fn nested_error(x: u8) -> NestedResult {
    Ok(if x > 3 { Err(String::from("nested limit")) } else { Ok(()) })
}

#[derive(Debug)]
struct Wrap<T>(T);
impl<T: Arbitrary> Arbitrary for Wrap<T> {
    fn arbitrary(g: &mut Gen) -> Self { Self(T::arbitrary(g)) }
}
#[derive(Debug)]
struct Leaf(u8);
#[test]
fn wrapped(x: Wrap<Leaf>) { assert!(x.0.0 < 3); }

#[derive(Debug)]
struct Many([u8; 512]);
impl Arbitrary for Many {
    fn arbitrary(g: &mut Gen) -> Self { Self(std::array::from_fn(|_| u8::try_from(g.index(16)).unwrap())) }
}

#[test]
fn budget(values: Many) {
    // Seed 4 draws a sum of 3958. Lowering any draw makes this case pass.
    assert!(values.0.iter().map(|n| *n as u32).sum::<u32>() < 3958);
}

mod outer {
    mod inner {
        use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

        #[derive(Debug)]
        struct Private<T, const N: usize> where T: Copy {
            field: [T; N],
        }
        #[derive(Debug)]
        enum Shape { None, Tuple(Private<u16, 2>), Named { on: bool } }

        type Input = Option<Shape>;
        type Bytes = [u8];

        #[derive(Debug)]
        struct Node { children: Vec<Node> }

        #[derive(Debug)]
        struct A<T> { b: Vec<B<T>>, c: Vec<C<T>> }
        #[derive(Debug)]
        struct B<T> { a: Vec<A<T>>, value: T }
        #[derive(Debug)]
        struct C<T> { b: Vec<B<T>> }

        type Text<'a> = &'a str;
        type MutText<'a> = &'a mut str;
        type Counter<'a> = &'a mut u32;
        type Shared<'a> = &'a Private<u8, 2>;
        type Slice<'a> = &'a [u8];
        type MutSlice<'a> = &'a mut [u8];

        #[test]
        pub fn aliases(text: Text<'_>, mutable_text: MutText<'_>, counter: Counter<'_>,
                   shared: Shared<'_>, grouped: (&str), slice: Slice<'_>, mutable_slice: MutSlice<'_>) {
            *counter = 0;
            mutable_text.make_ascii_uppercase();
            mutable_slice.fill(0);
            assert_eq!(shared.field.len(), 2);
            let _ = (text, grouped, slice);
        }

        #[test] fn recursive(value: A<u8>) { let _ = value; }
        #[test] fn variants(value: Shape) { assert!(!matches!(value, Shape::Named { .. })); }
        #[test] fn structs(value: Private<u8, 2>) { assert_eq!(value.field, [0, 0]); }
        #[test] fn options(value: Option<u8>) { assert!(value.is_none()); }
        #[test] fn errors(value: Result<u8, u8>) { assert!(value.is_ok()); }
        #[test] fn flags(value: bool) { assert!(!value); }
        #[test] fn characters(value: char) { assert!(value.is_ascii()); }
        #[test] fn strings(value: String) { assert!(value.is_empty()); }
        #[test] fn arrays(value: [u8; 2]) { assert_eq!(value, [0, 0]); }
        #[test] fn tuples(value: (u8, bool)) { assert_eq!(value, (0, false)); }
        #[test] fn floats(value: (f32, f64)) { assert!(value.0 == 0.0 && value.1 == 0.0); }
        #[test] fn maps(value: HashMap<u8, u8>) { assert!(value.is_empty()); }
        #[test] fn sets(value: HashSet<u8>) { assert!(value.is_empty()); }
        #[test] fn ordered_maps(value: BTreeMap<u8, u8>) { assert!(value.is_empty()); }
        #[test] fn ordered_sets(value: BTreeSet<u8>) { assert!(value.is_empty()); }

        #[test]
        fn all(
            value: Input, flag: bool, character: char, owned: String, text: &str,
            vector: Vec<(i8, i16, i32, i64, i128, isize)>, bytes: &mut Bytes,
            integers: [u128; 2], reference: &usize, mutable: &mut u32,
            number: Result<(f32, f64), (u16, u64)>,
            map: HashMap<u8, String>, set: HashSet<u8>,
            ordered: (BTreeMap<char, u8>, BTreeSet<u8>), node: Node,
        ) {
            if let Some(shape) = value {
                match shape {
                    Shape::None => {}
                    Shape::Tuple(item) => assert_eq!(item.field.len(), 2),
                    Shape::Named { on } => { let _ = on; }
                }
            }
            bytes.fill(0);
            *mutable = 0;
            assert!(map.keys().all(|key| map.contains_key(key)));
            assert!(set.iter().all(|key| set.contains(key)));
            assert!(node.children.len() <= 32);
            let _ = (flag, character, owned, text, vector, integers, reference, number, ordered);
        }
    }
}

mod separate;
"#,
    );
    std::fs::write(
        dir.join("package/src/separate.rs"),
        "#[derive(Debug)] struct Value(u8);\n#[test] fn separate(v: Value) { let _ = v.0; }",
    )
    .unwrap();
    for _ in 0..2 {
        checked(driver(&dir).args(["--crate-type=lib", "--test"]));
    }
    let executable = dir.join("out/demo-test");
    let listing = checked(Command::new(&executable).args(["--list", "--format=terse"]));
    let listing = String::from_utf8_lossy(&listing.stdout);
    for name in [
        "ordinary",
        "outer::inner::all",
        "separate::separate",
        "sum",
        "vector",
        "even",
    ] {
        assert!(
            listing.lines().any(|line| line == format!("{name}: test")),
            "{listing}"
        );
    }
    checked(Command::new(&executable).args(["--exact", "ordinary"]));
    checked(Command::new(&executable).args(["--exact", "separate::separate"]));
    checked(Command::new(&executable).args(["--exact", "outer::inner::aliases"]));
    checked(Command::new(&executable).args(["--exact", "outer::inner::recursive"]));
    for seed in [None, Some("27")] {
        let mut run = Command::new(&executable);
        run.args([
            "--exact",
            "outer::inner::all",
            "--test-threads=1",
            "--nocapture",
        ]);
        run.env_remove("TIDE_TEST_SEED");
        if let Some(seed) = seed {
            run.env("TIDE_TEST_SEED", seed);
        }
        let out = checked(&mut run);
        assert!(stderr(&out).contains(if seed.is_some() { "1/1" } else { "100/100" }));
    }

    let failure = |name: &str, seed: Option<&str>| {
        let mut run = Command::new(&executable);
        run.args(["--exact", name, "--test-threads=1"])
            .env_remove("TIDE_TEST_SEED")
            .stdin(Stdio::null());
        if let Some(seed) = seed {
            run.env("TIDE_TEST_SEED", seed);
        }
        let out = run.output().unwrap();
        assert!(!out.status.success(), "{out:?}");
        let output = format!("{}{}", String::from_utf8_lossy(&out.stdout), stderr(&out));
        let read = |prefix| {
            output
                .lines()
                .find_map(|line| line.trim().strip_prefix(prefix))
                .unwrap_or_else(|| panic!("missing {prefix} in {output}"))
                .to_owned()
        };
        let seed = read("seed: ");
        assert!(seed.parse::<u64>().is_ok());
        assert!(output.contains(&format!("tide test {name} --seed {seed}")));
        assert!(
            output.contains(&format!(
                "TIDE_TEST_SEED={seed} '{}' --exact {name}",
                executable.display()
            )),
            "{output}"
        );
        if name == "returned_error" || name == "nested_error" {
            assert!(!output.contains("Error:"), "{output}");
            assert_eq!(output.matches("cause: ").count(), 1, "{output}");
            assert!(
                output.contains(if name == "returned_error" {
                    "cause: \"limit\""
                } else {
                    "cause: \"nested limit\""
                }),
                "{output}"
            );
        }
        let budget = output.contains("shrink budget reached");
        (seed, read("input: "), budget)
    };
    let sum = failure("sum", None);
    assert_eq!(sum, failure("sum", None));
    assert_eq!(sum, failure("sum", Some(&sum.0)));
    let (a, b) = sum.1.split_once(", b = ").unwrap();
    let a: u16 = a.strip_prefix("a = ").unwrap().parse().unwrap();
    let b: u16 = b.parse().unwrap();
    assert_eq!(a + b, 301);
    assert_eq!(failure("vector", None).1, "xs = [100]");
    assert_eq!(failure("even", None).1, "x = Even(10)");
    assert_eq!(failure("same_failure", Some("4")).1, "x = 10");
    assert!(failure("budget", Some("4")).2);
    assert_eq!(failure("positive", Some("4")).1, "p = Positive(1)");
    assert_eq!(failure("returned_error", None).1, "x = 4");
    assert_eq!(failure("nested_error", None).1, "x = 4");
    assert_eq!(failure("wrapped", None).1, "x = Wrap(Leaf(3))");
    for family in [
        "variants",
        "structs",
        "options",
        "errors",
        "flags",
        "characters",
        "strings",
        "arrays",
        "tuples",
        "floats",
        "maps",
        "sets",
        "ordered_maps",
        "ordered_sets",
    ] {
        failure(&format!("outer::inner::{family}"), None);
    }
}

/// fx-m72.3: generated entries exercise contracts, skips, private-state sequences and vacuity.
#[test]
fn generated_contract_tests_execute_and_report_unchecked() {
    let dir = package(
        "generated-contracts",
        include_str!("../../../conformance/cases/contracts/generated_tests.rs"),
    );
    checked(driver(&dir).args(["--crate-type=lib", "--test", "--crate-name=meter"]));
    let binary = dir.join("out/meter-test");
    let listing = checked(Command::new(&binary).args(["--list", "--format=terse"]));
    let listing = String::from_utf8(listing.stdout).unwrap();
    let names: Vec<_> = listing
        .lines()
        .filter_map(|line| line.strip_suffix(": test"))
        .collect();
    assert!(names.contains(&"plain"), "{listing}");
    let run = |prefix: &str| {
        let name = names
            .iter()
            .find(|name| {
                name.rsplit("::")
                    .next()
                    .unwrap()
                    .starts_with(&format!("{prefix}_"))
            })
            .unwrap();
        let output = Command::new(&binary)
            .args(["--exact", name, "--nocapture"])
            .env_remove("TIDE_TEST_SEED")
            .stdin(Stdio::null())
            .output()
            .unwrap();
        let text = format!(
            "{}{}",
            String::from_utf8_lossy(&output.stdout),
            stderr(&output)
        );
        (output.status.success(), text)
    };
    for name in [
        "select",
        "identity",
        "string_bound",
        "capabilities",
        "unchanged",
        "constant",
        "paired_bounds",
        "repeated_caps",
        "public_nested",
        "seed_argument",
        "guided_custom",
        "custom_option",
        "custom_slice",
        "comment_clause",
        "multiline_literal",
        "capability_scope",
        "local_borrow",
        "projected_number",
        "argument_impl",
        "crate_substrings",
        "forged_const",
        "forged_runtime",
        "invariant_clause",
        "concrete_consts",
        "default_law_value",
    ] {
        let (ok, text) = run(name);
        assert!(
            ok && text.contains("100/100 cases passed"),
            "{name}: {text}"
        );
    }
    for (name, message) in [
        ("rebound", "ensures violated:"),
        ("phase", "ensures violated:"),
        ("ordinary_panic", "body fault"),
        ("forbidden", "vacuous:"),
        ("from_broken", "ensures violated:"),
        ("deeper_constructor", "requires violated:"),
        ("capability_vacuous", "vacuous:"),
    ] {
        let (ok, text) = run(name);
        assert!(!ok && text.contains(message), "{name}: {text}");
    }
    for (name, reason) in [
        ("allowed", "empty domain for this probe"),
        ("sealed", "constructor rejects every input"),
        ("cfg_allowed", "cfg attribute reason"),
    ] {
        let (ok, text) = run(name);
        assert!(
            ok && text.contains(&format!("unchecked (vacuous, allowed: {reason})")),
            "{name}: {text}"
        );
    }
    let (ok, text) = run("shrinking_seed");
    assert!(!ok && text.contains("input: seed = 1"), "{text}");
    for name in ["unmet", "closure_arg", "dynamic_arg", "handle_arg", "unavailable_constructor", "const_constructor"] {
        let (ok, text) = run(name);
        assert!(ok && text.contains("ignored, unchecked:"), "{name}: {text}");
    }
    for name in ["static_text", "static_alias", "bound_static", "transitive_static", "related_borrows", "nested_borrow", "trait_lifetime", "projected_borrow"] {
        let (ok, text) = run(name);
        assert!(ok && text.contains("ignored, unchecked:") && text.contains("lifetime"), "{name}: {text}");
    }
    for (name, reason) in [
        ("reexported", "type not nameable from the generated module"),
        ("local_method", "body-local function"),
    ] {
        let (ok, text) = run(name);
        assert!(ok && text.contains(&format!("ignored, unchecked: {reason}")), "{name}: {text}");
    }
    let entry = package("generated-entry", "#[ensures(true)] #[tide::main] fn main() {}");
    checked(driver(&entry).arg("--test"));
    let out = checked(Command::new(entry.join("out/demo-test")).arg("--nocapture"));
    assert!(String::from_utf8_lossy(&out.stdout).contains("ignored, unchecked: entry point"), "{}{}", String::from_utf8_lossy(&out.stdout), stderr(&out));

}

/// fx-m72.2: shared fixture executes guidance, rejection blame, and fixed-seed shrinking.
#[test]
fn guided_properties_filter_report_vacuity_and_shrink_simulation_parameters() {
    let dir = package(
        "guided-properties",
        include_str!("../../../conformance/cases/contracts/guided_properties.rs"),
    );
    checked(driver(&dir).args(["--crate-type=lib", "--test"]));
    let executable = dir.join("out/demo-test");
    let run = |name: &str, seed: Option<&str>, simulation: bool| {
        let mut command = Command::new(&executable);
        command
            .args(["--exact", &format!("properties::{name}"), "--nocapture"])
            .env_remove("TIDE_TEST_SEED")
            .env_remove("TIDE_SIM_SEED")
            .env_remove("__TIDE_SIM_CHILD");
        if let Some(seed) = seed {
            command.env(
                if simulation {
                    "TIDE_SIM_SEED"
                } else {
                    "TIDE_TEST_SEED"
                },
                seed,
            );
        }
        let out = command.stdin(Stdio::null()).output().unwrap();
        let text = format!("{}{}", String::from_utf8_lossy(&out.stdout), stderr(&out));
        (out.status.success(), text)
    };
    for name in [
        "index",
        "borrowed_index",
        "borrowed",
        "borrowed_shapes",
        "borrowed_scalars",
        "borrowed_string",
        "endpoints",
        "lengths",
        "shapes",
        "other_scalars",
        "shadowed",
        "grouped",
        "contains_one",
    ] {
        let (ok, text) = run(name, None, false);
        assert!(
            ok && text.contains("100/100 cases passed"),
            "{name}: {text}"
        );
    }
    for name in ["filter_always", "direct", "borrowed_constant"] {
        let (ok, text) = run(name, None, false);
        assert!(ok && text.contains("accepted/generated"), "{name}: {text}");
        assert!(!text.contains("100/100 cases passed"), "{name}: {text}");
    }
    for (name, clause, count) in [
        ("vacuous", "opaque(n)", "0 of 10000"),
        ("contradictory", "n < 0", "0 of 10000"),
        ("cyclic", "a < b", "0 of 10000"),
    ] {
        let (ok, text) = run(name, None, false);
        assert!(
            !ok && text.contains("vacuous:") && text.contains(count),
            "{text}"
        );
        assert!(
            text.contains(clause) && text.contains("most rejected by"),
            "{text}"
        );
    }
    let (ok, text) = run("expected", None, false);
    assert!(ok && !text.contains("vacuous"), "{text}");
    for name in ["nested", "nested_local", "nested_closure"] {
        let (ok, text) = run(name, None, false);
        assert!(
            !ok && text.contains("property failed:") && !text.contains("vacuous"),
            "{text}"
        );
    }
    let (ok, text) = run("shrink_filtered", None, false);
    assert!(!ok && text.contains("input: xs = [0, 0, 0]"), "{text}");
    for (name, input) in [
        ("shrink_index", "xs = [0, 0, 0, 0], i = 3"),
        ("shrink_span", "lo = 0, hi = 5"),
    ] {
        let (ok, text) = run(name, None, false);
        assert!(!ok && text.contains(&format!("input: {input}")), "{text}");
    }
    for name in ["simulated_pass", "simulated_struct", "simulated_guided_seed"] {
        let (ok, text) = run(name, None, true);
        assert!(
            ok && text.contains(if name == "simulated_pass" {
                "5/5 cases passed"
            } else {
                "2/2 cases passed"
            }),
            "{text}"
        );
    }
    let (ok, text) = run("simulated_filtered", None, true);
    assert!(
        ok && text.contains("5/") && text.contains("cases passed (accepted/generated)"),
        "{text}"
    );
    let (ok, text) = run("simulated_vacuous", None, true);
    assert!(
        !ok && text.contains("vacuous: 0 of 200") && text.contains("never(n)"),
        "{text}"
    );
    assert!(!text.contains("missing summary"), "{text}");
    let (ok, text) = run("simulated_generated_seed", None, true);
    assert!(!ok && text.contains("input: seed = 2"), "{text}");
    let out = Command::new(&executable)
        .args(["--exact", "properties::simulated", "--nocapture"])
        .env_remove("TIDE_TEST_SEED")
        .env("TIDE_SIM_SEED", "0")
        .env("__TIDE_SIM_CHILD", "demo::properties::simulated")
        .stdin(Stdio::null())
        .output()
        .unwrap();
    assert!(!out.status.success());
    let errors = stderr(&out);
    let evaluations: usize = errors
        .lines()
        .find_map(|line| line.trim().strip_prefix("shrink evaluations: "))
        .unwrap()
        .parse()
        .unwrap();
    assert!(evaluations > 0, "{errors}");
    assert_eq!(
        String::from_utf8_lossy(&out.stdout)
            .lines()
            .filter(|line| *line == "TIDE_SIM_SEED_START 0")
            .count(),
        evaluations + 2,
        "each initial, self-check and shrink evaluation must reset the backstop"
    );
    for (name, simulation) in [("simulated", true), ("explicit_seed", false)] {
        let (ok, text) = run(name, None, simulation);
        assert!(!ok && text.contains("input:"), "{text}");
        let seed = text
            .lines()
            .find_map(|line| line.trim().strip_prefix("seed: "))
            .unwrap();
        let input = text
            .lines()
            .find_map(|line| line.trim().strip_prefix("input: "))
            .unwrap();
        assert!(input.ends_with("sizes = [0, 0, 0]"), "{text}");
        if !simulation {
            assert!(input.starts_with(&format!("seed = {seed}, ")), "{text}");
        }
        let (ok, replay) = run(name, Some(seed), simulation);
        assert!(
            !ok && replay.contains(&format!("input: {input}")),
            "{replay}"
        );
        assert!(replay.contains(&format!("seed: {seed}")), "{replay}");
    }
    let dir = package(
        "simulation-struct-synthesis",
        r#"
mod private {
    #[derive(Debug)] struct Local { field: u8 }
    #[sim_test(seeds = 2)]
    fn structural(sim: std::sim::SimRoot, value: Local) { let _ = (sim, value.field); }
}
"#,
    );
    checked(driver(&dir).args(["--crate-type=lib", "--test"]));
    let out = checked(&mut Command::new(dir.join("out/demo-test")));
    assert!(String::from_utf8_lossy(&out.stdout).contains("1 passed"));
    let dir = package(
        "property-release-filter",
        r#"
fn even(n: u16) -> bool { n % 2 == 0 }
#[requires(n >= 9 && n <= 12 && even(n))]
#[test]
fn filtered(n: u16) { assert!(even(n)); }
#[test]
#[requires(n < 10 && n > 100)]
fn impossible(n: u16) { let _ = n; }
#[sim_test(seeds = 3)]
#[requires(even(n))]
fn simulated(sim: std::sim::SimRoot, n: u16) { let _ = sim; assert!(even(n)); }
"#,
    );
    checked(driver(&dir).args(["--crate-type=lib", "--test", "--tide-profile=release"]));
    for name in ["filtered", "simulated"] {
        let out = checked(Command::new(dir.join("out/demo-test")).args(["--exact", name]));
        assert!(String::from_utf8_lossy(&out.stdout).contains("1 passed"));
    }
    let out = Command::new(dir.join("out/demo-test"))
        .args(["--exact", "impossible", "--nocapture"])
        .stdin(Stdio::null())
        .output()
        .unwrap();
    assert!(
        !out.status.success() && stderr(&out).contains("vacuous:"),
        "{}",
        stderr(&out)
    );
}

#[test]
fn property_callbacks_cannot_capture_caller_state() {
    let dir = package(
        "property-capture",
        r#"
#[tide::main]
fn main() {
    let mut partial = 0;
    std::test::run_config("capture", std::test::Config::default(), |g| {
        let _ = g.index(2);
        partial += 1;
        Ok(())
    });
}
"#,
    );
    let output = driver(&dir).stdin(Stdio::null()).output().unwrap();
    assert_eq!(output.status.code(), Some(1));
    let errors = stderr(&output);
    assert!(
        errors.contains("expected fn pointer") && errors.contains("found closure"),
        "{errors}"
    );
    assert!(!dir.join("out/demo").exists());
}

#[test]
fn property_generation_diagnoses_unsupported_fields_and_attribute_arguments() {
    for (name, source, diagnostic, line) in [
        (
            "field",
            "#[derive(Debug)] struct Unsupported {\nfield: Box<u8>\n}\n#[test] fn check(missing: Box<u8>, x: Unsupported) {}",
            "field `field` of `Unsupported` has type `tide_std::boxed::Box<u8>`, which cannot be generated",
            "demo.rs:2:",
        ),
        (
            "slice",
            "#[derive(Debug)] struct Tail {\nfield: [u8]\n}\n#[test] fn check(x: &Tail) {}",
            "field `field` of `Tail` has type `[u8]`, which cannot be generated",
            "demo.rs:2:",
        ),
        (
            "requirements",
            "use std::test::{Arbitrary, Gen};\n\
             #[derive(Debug)] struct Forever<T>(T);\n\
             impl<T> Arbitrary for Forever<T> where Forever<Vec<T>>: Arbitrary {\n\
             fn arbitrary(_: &mut Gen) -> Self { panic!(\"no finite generator\"); } }\n\
             #[test] fn check(x: Forever<u8>) {}",
            "property generation requirements nest too deeply",
            "demo.rs:",
        ),
        (
            "attr",
            "#[test(args)] fn check() {}",
            "property tests take no attribute arguments",
            "demo.rs:1:",
        ),
        (
            "admission",
            "#[derive(Debug)] struct Value(u8); #[test] fn check(v: Value) { unsafe { let _ = v; } }",
            "TD0901",
            "demo.rs:1:",
        ),
        (
            "generated-vacuity-reason",
            "#[ensures(result)]\n#[allow(vacuous)]\nfn check() -> bool { true }",
            "allow(vacuous) needs a nonempty reason",
            "demo.rs:2:1",
        ),
        (
            "contract-admission",
            "#[requires({ let p = 0usize as *const u8; let _ = unsafe { *p }; true })]\n#[ensures(result == n)]\nfn identity(n: u8) -> u8 { n }",
            "TD0901",
            "demo.rs:1:",
        ),
        (
            "reserved-generator",
            "#[test]\n#[doc = \"__tide_draw:0:unsafe { 0u8 }\"]\nfn forged(n: u8) { let _ = n; }",
            "reserved property metadata is supplied only by tide-admit",
            "demo.rs:2:",
        ),
        (
            "reserved-capabilities",
            "#[test]\n#[doc = \"__tide_capabilities:\"]\nfn forged(n: u8) { let _ = n; }",
            "reserved property metadata is supplied only by tide-admit",
            "demo.rs:2:",
        ),
    ] {
        let dir = package(&format!("property-invalid-{name}"), source);
        let out = driver(&dir)
            .args(["--crate-type=lib", "--test"])
            .stdin(Stdio::null())
            .output()
            .unwrap();
        assert_eq!(out.status.code(), Some(1));
        let errors = stderr(&out);
        let diagnosed =
            errors.contains(diagnostic) || (name == "requirements" && errors.contains("E0275"));
        assert!(diagnosed && errors.contains(line), "{errors}");
        if name == "generated-vacuity-reason" {
            assert!(errors.lines().any(|text| text.contains(&format!("{line}: error: {diagnostic}"))), "{errors}");
        }
    }
}

#[test]
fn separate_library_properties_generate_suite_types_and_library_implemented_types() {
    let suite = r#"
use tide_candidate::{Level, Marker, Opaque, Wrap};

#[derive(Debug)]
struct Reading { level: Level, raw: u8 }

#[test]
fn levels_stay_in_range(reading: Reading) {
    assert!(reading.level.get() < 4, "{}", reading.raw);
}

#[test]
fn levels_are_low(reading: Reading) {
    assert!(reading.level.get() < 3);
}

#[test]
fn unused_type_argument(value: Marker<Opaque>) { let _ = value; }

#[derive(Debug)]
struct Leaf(u8);

#[test]
fn indirect_requirement(value: Wrap<Leaf>) { let _ = value.0.0; }
"#;
    let dir = package("separate-library-properties", suite);
    let candidate = dir.join("candidate/src/lib.rs");
    std::fs::create_dir_all(candidate.parent().unwrap()).unwrap();
    std::fs::write(
        &candidate,
        r#"
use std::test::{Arbitrary, Gen};

#[derive(Debug)]
pub struct Level(u8);
impl Level {
    pub fn get(&self) -> u8 { self.0 }
}
impl Arbitrary for Level {
    fn arbitrary(g: &mut Gen) -> Self { Level(u8::try_from(g.index(4)).unwrap()) }
}

#[derive(Debug)]
pub struct Opaque(pub u8);

#[derive(Debug)]
pub struct Marker<T>(std::marker::PhantomData<T>);
impl<T> Arbitrary for Marker<T> {
    fn arbitrary(_: &mut Gen) -> Self { Self(std::marker::PhantomData) }
}

pub trait Generate: Sized { fn generate(g: &mut Gen) -> Self; }
impl<T: Arbitrary> Generate for T {
    fn generate(g: &mut Gen) -> Self { T::arbitrary(g) }
}

#[derive(Debug)]
pub struct Wrap<T>(pub T);
impl<T: Generate> Arbitrary for Wrap<T> {
    fn arbitrary(g: &mut Gen) -> Self { Self(T::generate(g)) }
}
"#,
    )
    .unwrap();
    checked(&mut paired_test_driver(&dir));
    let test = dir.join("out/demo-test");
    let run = checked(Command::new(&test).args(["--exact", "levels_stay_in_range"]));
    assert!(String::from_utf8_lossy(&run.stdout).contains("1 passed"));
    checked(Command::new(&test).args(["--exact", "unused_type_argument"]));
    let run = checked(Command::new(&test).args(["--exact", "indirect_requirement"]));
    assert!(String::from_utf8_lossy(&run.stdout).contains("1 passed"));
    let run = Command::new(&test)
        .args(["--exact", "levels_are_low"])
        .stdin(Stdio::null())
        .output()
        .unwrap();
    assert!(!run.status.success());
    let report = format!(
        "{}{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert!(report.contains("level: Level(3)"), "{report}");

    std::fs::write(
        dir.join("package/src/demo.rs"),
        "use tide_candidate::Opaque;\n\
         #[derive(Debug)] struct Wrapper { inner: Opaque }\n\
         #[test] fn wrapped(w: Wrapper) { let _ = w.inner; }\n",
    )
    .unwrap();
    let out = paired_test_driver(&dir)
        .stdin(Stdio::null())
        .output()
        .unwrap();
    let diagnostic = stderr(&out);
    assert_eq!(out.status.code(), Some(1), "{diagnostic}");
    assert!(
        diagnostic.contains(
            "this property generates `tide_candidate::Opaque`, which the paired library defines \
             without implementing `std::test::Arbitrary`"
        ) && diagnostic.contains("demo.rs:3:"),
        "{diagnostic}"
    );
    assert!(!test.exists());
}

#[test]
fn property_synthesis_preserves_diagnostic_lines() {
    let source = "mod outer {\n\
                  mod inner {\n\
                  #[derive(Debug)] struct Private<T> { n: std::collections::BTreeMap<u8, (T, Vec<T>, Option<T>, Result<Vec<T>, (T, T)>, [T; 12])> }\n\
                  #[test] fn check(v: Private<u8>) { let _ = v.n; }\n\
                  }\n\
                  fn broken() { let _: u8 = \"wrong type\"; }\n\
                  }\n";
    let dir = package("property-diagnostic", source);
    let output = driver(&dir)
        .args(["--crate-type=lib", "--test"])
        .stdin(Stdio::null())
        .output()
        .unwrap();
    assert!(!output.status.success());
    let errors = stderr(&output);
    assert!(
        errors.contains("demo.rs:6:") && errors.contains("mismatched types"),
        "{errors}"
    );
    assert!(!errors.contains("Arbitrary"), "{errors}");
}

#[test]
fn overflow_traps_in_every_profile_in_binaries_and_tests() {
    let source = "fn increment(n: u8) -> u8 { n + 1 }\n\
        fn overflow() { let n = \"255\".parse::<u8>().unwrap(); println!(\"BEFORE_OVERFLOW\"); println!(\"{}\", increment(n)); }\n\
        #[tide::main] fn main() { overflow(); }\n\
        #[test] fn overflows() { overflow(); }\n";
    let dir = package("overflow", source);
    for profile in ["debug", "release"] {
        for test in [false, true] {
            let mut build = driver(&dir);
            build.arg(format!("--tide-profile={profile}"));
            if test {
                build.arg("--test");
            }
            let out = checked(&mut build);
            assert!(out.stdout.is_empty());
            let mut run = Command::new(dir.join(if test { "out/demo-test" } else { "out/demo" }));
            run.env_clear();
            if test {
                run.arg("--nocapture");
            }
            let out = run.stdin(Stdio::null()).output().unwrap();
            assert_eq!(
                out.status.code(),
                Some(101),
                "{profile}, test={test}: {}",
                stderr(&out)
            );
            assert!(
                stderr(&out).contains("attempt to add with overflow"),
                "{}",
                stderr(&out)
            );
            assert!(String::from_utf8_lossy(&out.stdout).contains("BEFORE_OVERFLOW"));
        }
    }
}

#[test]
fn cache_separates_metadata_link_profiles_and_test_configuration() {
    let dir = package(
        "cache",
        "#[tide::main] fn main() {}\n#[test] fn passes() {}\n",
    );
    let entries = || {
        std::fs::read_dir(dir.join("out/.tide-cache"))
            .unwrap()
            .count()
    };
    checked(driver(&dir).arg("--emit=metadata"));
    assert_eq!(entries(), 1);
    assert!(!dir.join("out/demo").exists());
    for profile in ["debug", "release"] {
        for test in [false, true] {
            let mut build = driver(&dir);
            build.arg(format!("--tide-profile={profile}"));
            if test {
                build.arg("--test");
            }
            checked(&mut build);
        }
    }
    assert_eq!(entries(), 5);
    let before: Vec<_> = std::fs::read_dir(dir.join("out/.tide-cache"))
        .unwrap()
        .map(|e| {
            let e = e.unwrap();
            (e.path(), e.metadata().unwrap().modified().unwrap())
        })
        .collect();
    std::fs::remove_file(dir.join("out/demo-test")).unwrap();
    checked(driver(&dir).args(["--test", "--tide-profile=release"]));
    assert_eq!(entries(), 5);
    for (path, modified) in before {
        assert_eq!(path.metadata().unwrap().modified().unwrap(), modified);
    }
    checked(&mut Command::new(dir.join("out/demo-test")));
}

#[test]
fn rejected_source_never_links_or_runs_and_removes_stale_outputs() {
    let valid = "#[tide::main] fn main() {}\n#[test] fn passes() {}\n";
    let dir = package("reject", valid);
    for (source, flags, executable, code) in [
        (
            "#[tide::main] fn main() { println!(\"REJECTED_RAN\"); let _ = std::__rt::enter(|_| ()); }",
            vec![],
            "demo",
            "TD0520",
        ),
        (
            "fn f() {}\n#[tide::main] fn main() { println!(\"REJECTED_RAN {:?}\", f as fn()); }",
            vec!["--tide-test-backstop"],
            "demo",
            "TD0520",
        ),
        (
            "#[tide::main] fn main() { println!(\"REJECTED_RAN\"); }\n#[test] fn calls_entry() { main(); }",
            vec!["--test"],
            "demo-test",
            "TD0506",
        ),
        (
            "fn main() { println!(\"REJECTED_RAN\"); }",
            vec![],
            "demo",
            "TD0512",
        ),
        (
            "core::arch::global_asm!(\".error \\\"ASSEMBLER_RAN\\\"\");\nfn need<T: Pure>(_: T) {}\n#[tide::main] fn main(root: Root) { need(root.clock); }",
            vec!["--tide-grant=unsafe", "--tide-test-backstop"],
            "demo",
            "TD0517",
        ),
        (
            "unsafe extern \"C\" { safe fn tide_missing_link_symbol_for_test(); }\n#[tide::main] fn main() { tide_missing_link_symbol_for_test(); }",
            vec!["--tide-grant=unsafe"],
            "demo",
            "linking with",
        ),
    ] {
        std::fs::write(dir.join("package/src/demo.rs"), valid).unwrap();
        checked(driver(&dir).args(&flags));
        assert!(dir.join("out").join(executable).exists());
        let count = std::fs::read_dir(dir.join("out/.tide-cache"))
            .unwrap()
            .count();
        std::fs::write(dir.join("package/src/demo.rs"), source).unwrap();
        let out = driver(&dir).args(&flags).output().unwrap();
        assert_eq!(out.status.code(), Some(1), "{}", stderr(&out));
        assert!(stderr(&out).contains(code), "{}", stderr(&out));
        assert!(!stderr(&out).contains("ASSEMBLER_RAN"), "{}", stderr(&out));
        assert!(out.stdout.is_empty());
        assert!(!dir.join("out").join(executable).exists());
        assert!(!dir.join("out/demo.interface.json").exists());
        assert_eq!(
            std::fs::read_dir(dir.join("out/.tide-cache"))
                .unwrap()
                .count(),
            count
        );
    }
}

#[test]
fn granted_native_inputs_are_rebuilt_without_cache_reuse() {
    let dir = package("native-input", "");
    let input = dir.join("package/src/payload.bin");
    let assembly = format!(
        ".pushsection .rodata\n.global tide_payload\ntide_payload:\n.incbin {input:?}\n.popsection"
    );
    let source = format!(
        "core::arch::global_asm!({assembly:?});\nunsafe extern \"C\" {{ static tide_payload: u8; }}\n#[tide::main] fn main() {{\n // SAFETY: the linked assembly defines tide_payload as an initialized byte.\n println!(\"{{}}\", unsafe {{ tide_payload }}); }}"
    );
    std::fs::write(dir.join("package/src/demo.rs"), source).unwrap();
    for byte in [1u8, 2] {
        std::fs::write(&input, [byte]).unwrap();
        checked(driver(&dir).arg("--tide-grant=unsafe"));
        let out = checked(&mut Command::new(dir.join("out/demo")));
        assert_eq!(out.stdout, format!("{byte}\n").as_bytes());
        assert_eq!(
            std::fs::read_dir(dir.join("out/.tide-cache"))
                .unwrap()
                .count(),
            0
        );
    }
}

#[test]
fn link_mode_keeps_the_source_loader_boundary() {
    let dir = package("loader", "");
    let canary = dir.join("private.rs");
    std::fs::write(&canary, "PRIVATE_LINK_CANARY").unwrap();
    std::fs::write(
        dir.join("package/src/demo.rs"),
        format!(
            "#[tide::main] fn main() {{ println!(\"{{}}\", core::include_str!({canary:?})); }}"
        ),
    )
    .unwrap();
    let out = driver(&dir).output().unwrap();
    assert_eq!(out.status.code(), Some(1), "{}", stderr(&out));
    assert!(!stderr(&out).contains("PRIVATE_LINK_CANARY"));
    assert!(!dir.join("out/demo").exists());
    assert_eq!(
        std::fs::read_dir(dir.join("out/.tide-cache"))
            .unwrap()
            .count(),
        0
    );
}

// fx-5vj: process I/O executes through admission against the shared production std.
#[test]
fn process_inputs_execute_real_and_simulated() {
    use std::io::Write;
    let dir = package("process-inputs", include_str!("../../../conformance/cases/authority/process_inputs.rs"));
    checked(&mut driver(&dir));
    let mut child = Command::new(dir.join("out/demo"))
        .args(["one", "two words"]).env_clear().env("TIDE_INPUT", "value")
        .stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn().unwrap();
    child.stdin.take().unwrap().write_all(b"one\ntwo\r\nthree\n").unwrap();
    let out = child.wait_with_output().unwrap();
    assert!(out.status.success(), "{}", stderr(&out));
    assert_eq!(out.stdout, b"ab\nd\n");
    assert_eq!(out.stderr, b"c\ne\ninfo: ready\n");
    checked(driver(&dir).args(["--crate-type=lib", "--test"]));
    let first = checked(Command::new(dir.join("out/demo-test")).args(["simulated_", "--nocapture"]));
    let second = checked(Command::new(dir.join("out/demo-test")).args(["simulated_", "--nocapture"]));
    assert_eq!(first.stderr, b"");
    // libtest timing varies; the test compares full observations and emits no captured bytes.
    for out in [first, second] {
        let text = String::from_utf8(out.stdout).unwrap();
        assert!(text.contains("2 passed; 0 failed"), "{text}");
        assert!(!text.contains("info: ready"), "{text}");
    }
}

#[test]
fn process_main_exit_codes_and_sink_order() {
    for (name, result, body, code, error) in [
        ("unit", "()", "()", 0, ""),
        ("err", "Result<(), &'static str>", "Err(\"bad\")", 1, "Error: \"bad\"\n"),
        ("exit", "std::process::ExitCode", "std::process::ExitCode::from(7)", 7, ""),
        ("custom", "Custom", "Custom", 9, ""),
        ("panic", "()", "panic!(\"deliberate\")", 101, "deliberate"),
    ] {
        let custom = if name == "custom" {
            "struct Custom; impl std::process::Termination for Custom { fn report(self) -> std::process::ExitCode { std::process::ExitCode::from(9) } }"
        } else { "" };
        let dir = package(&format!("process-main-{name}"), &format!(r#"
{custom}
#[tide::main]
fn main() -> {result} {{
    use std::io::Write;
    print!("first");
    std::io::stdout().write_all(b" second\n").unwrap();
    eprint!("third ");
    std::io::stderr().write_all(b"fourth\n").unwrap();
    {body}
}}
"#));
        checked(&mut driver(&dir));
        let out = Command::new(dir.join("out/demo")).stdin(Stdio::null()).output().unwrap();
        assert_eq!(out.status.code(), Some(code), "{}", stderr(&out));
        assert_eq!(out.stdout, b"first second\n");
        if name == "panic" { assert!(stderr(&out).contains(error)); }
        else { assert_eq!(stderr(&out), format!("third fourth\n{error}")); }
        // One descriptor proves ordering across out, err, and Termination's own diagnostic.
        let file = std::fs::File::create(dir.join("merged")).unwrap();
        let status = Command::new(dir.join("out/demo")).stdin(Stdio::null())
            .stdout(file.try_clone().unwrap()).stderr(file).status().unwrap();
        assert_eq!(status.code(), Some(code));
        let merged = std::fs::read_to_string(dir.join("merged")).unwrap();
        assert!(merged.starts_with("first second\nthird fourth\n"), "{merged}");
    }
}

#[test]
fn process_stdin_reader_and_log_config_remain_nonpure() {
    let dir = package("process-purity", r#"
fn pure_value(_: impl std::marker::Pure) {}
#[tide::main]
fn main(root: Root) {
    pure_value(root.stdin.lines());
    pure_value(root.log);
}
"#);
    let out = driver(&dir).stdin(Stdio::null()).output().unwrap();
    assert!(!out.status.success());
    assert!(stderr(&out).contains("TD0517"), "{}", stderr(&out));
    assert!(stderr(&out).contains("StdinReader"), "{}", stderr(&out));
    assert!(stderr(&out).contains("LogConfig"), "{}", stderr(&out));
}

#[test]
fn process_logs_format_filter_redirect_and_warn_without_payload() {
    let dir = package("process-logs", include_str!("../../../conformance/cases/authority/process_sinks.rs"));
    checked(&mut driver(&dir));
    let out = checked(Command::new(dir.join("out/demo")).env("LOG_FILE", dir.join("log")));
    assert_eq!(out.stdout, b"done\n");
    assert_eq!(stderr(&out), "info: default 1\nerror: error\nwarn: warn\ndebug: debug\ntrace: trace\n{\"level\":\"info\",\"message\":\"quote=\\\" newline=\\n café\"}\n");
    let file = std::fs::read_to_string(dir.join("log")).unwrap();
    assert_eq!(file, "{\"level\":\"info\",\"message\":\"file\"}\n{\"level\":\"warn\",\"message\":\"BufWriter dropped with 14 unflushed bytes\"}\n");
}

#[test]
fn process_nonunicode_inputs_panic_lazily_and_var_preserves_os_value() {
    use std::os::unix::ffi::OsStringExt;
    let bad = std::ffi::OsString::from_vec(vec![0xff]);
    let dir = package("process-unicode", r#"
#[tide::main]
fn main(root: Root) {
    let mut args = root.env.args();
    assert!(args.next().is_some());
    let mode = args.next().unwrap();
    if mode == "args" {
        println!("before");
        args.next();
    } else {
        assert!(matches!(root.env.var("BAD"), Err(std::env::VarError::NotUnicode(_))));
        let mut vars = root.env.vars();
        println!("before");
        if mode == "vars" { vars.next(); }
    }
}
"#);
    checked(&mut driver(&dir));
    for (mode, code) in [("args", 101), ("vars", 101), ("var", 0)] {
        let out = Command::new(dir.join("out/demo")).env_clear().env("BAD", &bad)
            .arg(mode).arg(&bad).stdin(Stdio::null()).output().unwrap();
        assert_eq!(out.status.code(), Some(code), "{}", stderr(&out));
        assert_eq!(out.stdout, b"before\n");
    }
}

#[test]
fn process_stdin_boundaries_and_cancellation_execute_on_both_backends() {
    use std::io::Write;
    let dir = package("process-stdin", r#"
use std::io::{self, Read, Stdin};
use std::task::{Tasks, TaskError};
fn exercise(stdin: &Stdin, tasks: &Tasks) {
    let mut reader = stdin;
    tasks.scope::<_, TaskError, _>(|scope| {
        scope.cancel();
        assert!(reader.read(&mut [0; 2]).unwrap_err().is_cancelled());
        assert!(stdin.read_line(&mut String::new()).unwrap_err().is_cancelled());
        let mut lines = stdin.lines();
        assert!(lines.next().unwrap().unwrap_err().is_cancelled());
        assert!(lines.next().is_none());
        Ok(())
    }).unwrap();
    assert_eq!(reader.read(&mut []).unwrap(), 0);
    let mut prefix = [0; 2];
    assert_eq!(reader.read(&mut prefix).unwrap(), 2);
    assert_eq!(prefix, *b"zz");
    let mut lines = stdin.lines();
    assert_eq!(lines.next().unwrap().unwrap(), "a");
    assert_eq!(lines.next().unwrap().unwrap(), "b");
    assert_eq!(lines.next().unwrap().unwrap_err().kind(), io::ErrorKind::InvalidData);
    drop(lines);
    let mut invalid = String::from("kept");
    assert_eq!(stdin.read_line(&mut invalid).unwrap_err().kind(), io::ErrorKind::InvalidData);
    assert_eq!(invalid, "kept");
    assert_eq!(stdin.lines().next().unwrap().unwrap(), "c");
    let mut rest = vec![b'x'];
    assert_eq!(stdin.read_to_end(&mut rest).unwrap(), 4);
    assert_eq!(rest, b"xtail");
    assert_eq!(stdin.read_line(&mut String::new()).unwrap(), 0);
    assert_eq!(stdin.clone().read(&mut [0; 1]).unwrap(), 0);
}
#[tide::main]
fn main(root: Root) { exercise(&root.stdin, &root.tasks); }
#[test]
fn simulated() {
    let sim = Root::sim(2);
    sim.world.stdin(b"zza\nb\r\n\xff\n\xff\nc\ntail");
    exercise(&sim.stdin, &sim.tasks);
    assert_eq!(sim.elapsed(), std::time::Duration::ZERO);
}
"#);
    checked(&mut driver(&dir));
    let mut child = Command::new(dir.join("out/demo")).stdin(Stdio::piped())
        .stdout(Stdio::piped()).stderr(Stdio::piped()).spawn().unwrap();
    child.stdin.take().unwrap().write_all(b"zza\nb\r\n\xff\n\xff\nc\ntail").unwrap();
    let out = child.wait_with_output().unwrap();
    assert!(out.status.success(), "{}", stderr(&out));
    assert_eq!(out.stdout, b"");
    checked(driver(&dir).args(["--crate-type=lib", "--test"]));
    checked(Command::new(dir.join("out/demo-test")).args(["--exact", "simulated"]));
}

#[test]
fn process_closed_stdout_exits_141_and_concurrent_records_stay_whole() {
    use std::io::BufRead;
    let dir = package("process-sink-threads", r#"
#[tide::main]
fn main(root: Root) {
    root.tasks.scope::<_, std::task::TaskError, _>(|scope| {
        for worker in 0..4 {
            scope.spawn(move || {
                for _ in 0..1000 { println!("{}:{}", worker, "x".repeat(200)); }
                Ok(())
            });
        }
        Ok(())
    }).unwrap();
}
"#);
    checked(&mut driver(&dir));
    let out = checked(&mut Command::new(dir.join("out/demo")));
    let text = String::from_utf8(out.stdout).unwrap();
    assert_eq!(text.lines().count(), 4000);
    for line in text.lines() {
        assert_eq!(line.len(), 202);
        assert!(line.ends_with(&"x".repeat(200)), "{line}");
    }
    let mut child = Command::new(dir.join("out/demo")).stdin(Stdio::null())
        .stdout(Stdio::piped()).stderr(Stdio::piped()).spawn().unwrap();
    let mut pipe = std::io::BufReader::new(child.stdout.take().unwrap());
    let mut first = String::new();
    pipe.read_line(&mut first).unwrap();
    drop(pipe);
    let out = child.wait_with_output().unwrap();
    assert_eq!(out.status.code(), Some(141));
    assert_eq!(out.stderr, b"");
}

/// std::io's core (bead fx-ecb) passes the driver and runs: an in-memory reader through
/// `BufReader` and `BufRead::lines`, writes through `Write` and `BufWriter`, `Cursor` and
/// `SeekFrom`, an `io::Error` from an `ErrorKind` through `?`, and `is_cancelled()`.
#[test]
fn io_core_builds_and_runs() {
    let source = r#"
use std::io::{self, BufRead, BufReader, BufWriter, Cursor, Read, Seek, SeekFrom, Write};

fn numbered(input: impl Read, out: &mut impl Write) -> io::Result<usize> {
    let mut words = 0;
    for (i, line) in BufReader::new(input).lines().enumerate() {
        let line = line?;
        words += line.split_whitespace().count();
        writeln!(out, "{}: {line}", i + 1)?;
    }
    out.flush()?;
    Ok(words)
}

fn find(found: bool) -> io::Result<()> {
    if found { Ok(()) } else { Err(io::ErrorKind::NotFound.into()) }
}

#[tide::main]
fn main() -> io::Result<()> {
    let mut out = Vec::new();
    let words = numbered(&b"alpha beta\r\ngamma\n"[..], &mut out)?;
    print!("{}", String::from_utf8(out).unwrap());

    let mut cursor = Cursor::new(Vec::new());
    {
        let mut w = BufWriter::new(&mut cursor);
        w.write_all(b"xyz")?;
        w.flush()?;
    }
    cursor.seek(SeekFrom::Start(1))?;
    let mut rest = String::new();
    cursor.read_to_string(&mut rest)?;

    let missing = find(false).unwrap_err();
    let cancelled = io::Error::from(io::ErrorKind::Cancelled);
    let custom = io::Error::new(io::ErrorKind::InvalidData, "bad header");
    println!(
        "words={words} rest={rest} missing={missing} {:?} cancelled={} {} custom={custom}",
        missing.kind(),
        missing.is_cancelled(),
        cancelled.is_cancelled(),
    );
    Ok(())
}
"#;
    let dir = package("io-core", source);
    let out = checked(&mut driver(&dir));
    assert!(out.stderr.is_empty(), "{}", stderr(&out));
    let out = checked(&mut Command::new(dir.join("out/demo")));
    assert_eq!(
        String::from_utf8_lossy(&out.stdout),
        "1: alpha beta\n2: gamma\nwords=3 rest=yz missing=entity not found NotFound \
         cancelled=false true custom=bad header\n"
    );
}

#[test]
fn numeric_conversions_build_and_execute_against_production_std() {
    let source = include_str!("../../../conformance/cases/authority/numeric_conversions.rs");
    let dir = package("numeric-conversions", source);
    for profile in ["debug", "release"] {
        let build = checked(driver(&dir).arg(format!("--tide-profile={profile}")));
        assert!(!stderr(&build).contains("warning:"), "{}", stderr(&build));
        let out = checked(&mut Command::new(dir.join("out/demo")));
        assert_eq!(out.stdout, b"NUMERIC_CONVERSIONS_RAN\n");
        assert!(out.stderr.is_empty(), "{}", stderr(&out));
        checked(driver(&dir).args(["--test", &format!("--tide-profile={profile}")]));
        let out = checked(&mut Command::new(dir.join("out/demo-test")));
        assert!(String::from_utf8_lossy(&out.stdout).contains("1 passed"));
    }
}

#[test]
fn numeric_conversion_traits_keep_the_recorded_pair_sets() {
    for (name, statement) in [
        ("cross-sign", "let _: u8 = (-1i16).truncate();"),
        ("to-pointer", "let _: usize = 1u64.truncate();"),
        ("from-pointer", "let _: u32 = 1usize.truncate();"),
        ("exact-round", "let _ = f64::round_from(1u8);"),
    ] {
        let dir = package(
            &format!("numeric-conversions-reject-{name}"),
            &format!("#[tide::main] fn main() {{ {statement} }}"),
        );
        let out = driver(&dir).stdin(Stdio::null()).output().unwrap();
        assert_eq!(out.status.code(), Some(1), "{}", stderr(&out));
        assert!(stderr(&out).contains("E0277"), "{}", stderr(&out));
        assert!(!dir.join("out/demo").exists());
    }
}

#[test]
fn float_math_builds_and_executes_against_production_std() {
    let source = include_str!("../../../conformance/cases/authority/float_math.rs");
    let dir = package("float-math", source);
    for profile in ["debug", "release"] {
        checked(driver(&dir).arg(format!("--tide-profile={profile}")));
        let out = checked(&mut Command::new(dir.join("out/demo")));
        assert_eq!(out.stdout, b"FLOAT_MATH_RAN\n");
        assert!(out.stderr.is_empty(), "{}", stderr(&out));
        checked(driver(&dir).args(["--test", &format!("--tide-profile={profile}")]));
        let out = checked(&mut Command::new(dir.join("out/demo-test")));
        assert!(String::from_utf8_lossy(&out.stdout).contains("2 passed"));
    }
}

#[test]
fn float_banned_methods_still_report_td0520_with_production_std() {
    // Each rejection is isolated so one diagnostic cannot conceal an unguarded method.
    for ty in ["f32", "f64"] {
        for (method, args) in [
            ("to_bits", ""),
            ("is_sign_positive", ""),
            ("is_sign_negative", ""),
            ("copysign", "x"),
            ("signum", ""),
            ("total_cmp", "&x"),
            ("min", "x"),
            ("max", "x"),
            ("sin", ""),
            ("cos", ""),
            ("tan", ""),
            ("asin", ""),
            ("acos", ""),
            ("atan", ""),
            ("sinh", ""),
            ("cosh", ""),
            ("tanh", ""),
            ("asinh", ""),
            ("acosh", ""),
            ("atanh", ""),
            ("exp", ""),
            ("exp2", ""),
            ("exp_m1", ""),
            ("ln", ""),
            ("ln_1p", ""),
            ("log2", ""),
            ("log10", ""),
            ("cbrt", ""),
            ("round_ties_even", ""),
            ("powf", "x"),
            ("hypot", "x"),
            ("atan2", "x"),
            ("abs_sub", "x"),
            ("powi", "3"),
            ("sin_cos", ""),
            ("log", "x"),
            ("fract", ""),
            ("recip", ""),
            ("to_degrees", ""),
            ("to_radians", ""),
            ("div_euclid", "x"),
            ("rem_euclid", "x"),
            ("midpoint", "x"),
            ("clamp", "0.0, 1.0"),
        ] {
            let dir = package(
                &format!("float-reject-{ty}-{method}"),
                &format!(
                    "#[tide::main] fn main() {{ let x = 0.5{ty}; let _ = x.{method}({args}); }}"
                ),
            );
            let out = driver(&dir)
                .arg("--emit=metadata")
                .stdin(Stdio::null())
                .output()
                .unwrap();
            assert_eq!(
                out.status.code(),
                Some(1),
                "{ty}::{method}: {}",
                stderr(&out)
            );
            assert!(
                stderr(&out).contains("TD0520:"),
                "{ty}::{method}: {}",
                stderr(&out)
            );
            assert!(
                stderr(&out).contains(method),
                "{ty}::{method}: {}",
                stderr(&out)
            );
        }
    }
}

#[test]
fn error_derive_builds_and_executes_against_production_std() {
    let source = include_str!("../../../conformance/cases/errors/derive.rs");
    let dir = package("error-derive", source);
    checked(&mut driver(&dir));
    let out = checked(&mut Command::new(dir.join("out/demo")));
    assert_eq!(out.stdout, b"ERROR_DERIVE_RAN\n");
    checked(driver(&dir).args(["--crate-type=lib", "--test"]));
    let out = checked(&mut Command::new(dir.join("out/demo-test")));
    assert!(String::from_utf8_lossy(&out.stdout).contains("1 passed"));
}

/// `derive(Encode, Decode)` and `std::json` (bead fx-08m) pass the driver and execute: round
/// trips of structs, enums, `Option`, `Vec`, and maps, each `#[codec]` key, missing and unknown
/// keys, malformed input, float and integer limits, and a map keyed from `Rand`.
#[test]
fn codec_derives_and_json_build_and_execute_against_production_std() {
    let source = include_str!("../../../conformance/cases/codec/json.rs");
    let dir = package("codec-json", source);
    let out = checked(&mut driver(&dir));
    assert!(out.stderr.is_empty(), "{}", stderr(&out));
    let out = checked(&mut Command::new(dir.join("out/demo")));
    assert_eq!(out.stdout, b"CODEC_RAN\n");
    checked(driver(&dir).args(["--crate-type=lib", "--test"]));
    let out = checked(&mut Command::new(dir.join("out/demo-test")));
    assert!(String::from_utf8_lossy(&out.stdout).contains("1 passed"));
}

#[test]
fn codec_recursive_newtype_returns_error_in_subprocess() {
    let source = r#"
#[derive(Decode)]
struct Link(Option<Box<Link>>);

#[tide::main]
fn main(_root: Root) {
    assert!(std::json::decode::<Link>(b"null").unwrap().0.is_none());
    let error = std::json::decode::<Link>(b"0").err().unwrap();
    assert!(error.to_string().starts_with("recursion limit exceeded"));
}
"#;
    let dir = package("codec-recursive-newtype", source);
    checked(&mut driver(&dir));
    checked(Command::new("timeout").arg("10").arg(dir.join("out/demo")));
}

/// Each case's errors are all on `line`, and one of them contains `message`.
#[test]
fn codec_derives_reject_unsupported_input_at_its_line() {
    for (name, source, message, line) in [
        (
            "variant_skip",
            "#[derive(Encode)]\nenum Bad {\n    #[codec(skip)]\n    A,\n}",
            "`skip` is not allowed here, since `encode` cannot fail",
            3,
        ),
        (
            "unknown_key",
            "#[derive(Encode)]\nstruct Bad {\n    #[codec(skip_serializing_if = \"Option::is_none\")]\n    a: Option<u8>,\n}",
            "unknown `#[codec]` key; `#[codec]` takes `rename = \"..\"`, `rename_all = \"..\"`, `default`, and `skip`",
            3,
        ),
        (
            "not_in_v0",
            "#[derive(Decode)]\n#[codec(deny_unknown_fields)]\nstruct Bad { a: u8 }",
            "`deny_unknown_fields` is not in v0",
            2,
        ),
        (
            "default_path",
            "#[derive(Decode)]\nstruct Bad {\n    #[codec(default = \"make\")]\n    a: u8,\n}",
            "`default` takes no value",
            3,
        ),
        (
            "rename_forms",
            "#[derive(Decode)]\nstruct Bad {\n    #[codec(rename(deserialize = \"b\"))]\n    a: u8,\n}",
            "expected `=`",
            3,
        ),
        (
            "field_collision",
            "#[derive(Encode)]\nstruct Bad {\n    a: u8,\n    #[codec(rename = \"a\")]\n    b: u8,\n}",
            "two fields have the JSON name `a`",
            5,
        ),
        (
            "variant_collision",
            "#[derive(Decode)]\n#[codec(rename_all = \"lowercase\")]\nenum Bad {\n    Ab,\n    AB,\n}",
            "two variants have the JSON name `ab`",
            5,
        ),
        (
            "case_name",
            "#[derive(Encode)]\n#[codec(rename_all = \"Title Case\")]\nstruct Bad { a: u8 }",
            "expected one of \"lowercase\", \"UPPERCASE\", \"PascalCase\", \"camelCase\", \"snake_case\", \"SCREAMING_SNAKE_CASE\", \"kebab-case\", \"SCREAMING-KEBAB-CASE\"",
            2,
        ),
        (
            "variant_rename_all",
            "#[derive(Encode)]\nenum Bad {\n    #[codec(rename_all = \"camelCase\")]\n    A { long_name: u8 },\n}",
            "`rename_all` is not allowed here; it goes on the struct or enum",
            3,
        ),
        (
            "container_rename",
            "#[derive(Encode)]\n#[codec(rename = \"Good\")]\nstruct Bad { a: u8 }",
            "`rename` is not allowed here; it goes on a field or a variant",
            2,
        ),
        (
            "enum_default",
            "#[derive(Decode)]\n#[codec(default)]\nenum Bad { A }",
            "`default` is not allowed here; it goes on a field or a struct",
            2,
        ),
        (
            "tuple_field",
            "#[derive(Decode)]\nstruct Bad(\n    #[codec(default)]\n    u8,\n    u8,\n);",
            "`#[codec]` is not allowed on a tuple field",
            3,
        ),
        (
            "unit_default",
            "#[derive(Decode)]\n#[codec(default)]\nstruct Bad;",
            "`default` is not allowed here; it goes on a field or a struct with named fields or an array representation",
            2,
        ),
        (
            "newtype_default",
            "#[derive(Encode)]\n#[codec(default)]\nstruct Bad(u8);",
            "`default` is not allowed here; it goes on a field or a struct with named fields or an array representation",
            2,
        ),
        (
            "duplicate_key",
            "#[derive(Encode)]\nstruct Bad {\n    #[codec(rename = \"a\")]\n    #[codec(rename = \"b\")]\n    c: u8,\n}",
            "duplicate `rename`",
            4,
        ),
        (
            "union",
            "#[derive(Encode)]\nunion Bad { a: u8 }",
            "cannot be derived for a union",
            2,
        ),
        // ROOT-4: a capability has no `Decode`, and `skip` needs the `Default` it also lacks.
        (
            "capability",
            "#[derive(Decode)]\nstruct Bad {\n    net: std::net::Net,\n}",
            "Decode",
            3,
        ),
        (
            "capability_skip",
            "#[derive(Decode)]\nstruct Bad {\n    #[codec(skip)]\n    net: std::net::Net,\n}",
            "Default",
            4,
        ),
        // Decoding is owned: a borrowed field encodes only.
        (
            "borrowed",
            "#[derive(Encode, Decode)]\nstruct Bad<'a> {\n    name: &'a str,\n}",
            "Decode",
            3,
        ),
        (
            "non_string_key",
            "use std::collections::BTreeMap;\n#[derive(Encode)]\nstruct Bad {\n    by_id: BTreeMap<u32, u8>,\n}",
            "Encode",
            4,
        ),
        // `skip` needs `Default` for `Decode` only.
        (
            "missing_default",
            "struct Plain;\n#[derive(Encode)]\nstruct Fine {\n    #[codec(skip)]\n    a: Plain,\n}\n#[derive(Decode)]\nstruct Bad {\n    #[codec(skip)]\n    a: Plain,\n}",
            "Default",
            10,
        ),
    ] {
        let dir = package(&format!("codec-negative-{name}"), source);
        let out = driver(&dir)
            .args([
                "--crate-type=lib",
                "--emit=metadata",
                "--error-format=short",
            ])
            .output()
            .unwrap();
        assert_eq!(out.status.code(), Some(1), "{name}: {}", stderr(&out));
        let err = stderr(&out);
        let errors: Vec<_> = err.lines().filter(|l| l.contains(": error")).collect();
        assert!(
            errors
                .iter()
                .all(|l| l.contains(&format!("demo.rs:{line}:")))
                && errors.iter().any(|l| l.contains(message)),
            "{name}: {err}"
        );
    }
}

#[test]
fn error_derive_rejects_malformed_helpers_at_the_input() {
    for (name, source, message, line) in [
        (
            "literal",
            "#[derive(Debug, Error)]\n#[error(12)]\nstruct Bad;",
            "expected a string literal or `transparent`",
            2,
        ),
        (
            "duplicate",
            "#[derive(Debug, Error)]\n#[error(\"one\")]\n#[error(\"two\")]\nstruct Bad;",
            "duplicate #[error]",
            3,
        ),
        (
            "transparent",
            "#[derive(Debug, Error)]\n#[error(transparent)]\nstruct Bad(u8, u8);",
            "requires exactly one field",
            2,
        ),
        (
            "from",
            "#[derive(Debug, Error)]\n#[error(\"bad\")]\nstruct Bad(#[from] std::io::Error, u8);",
            "no fields other than the source",
            3,
        ),
        (
            "sources",
            "#[derive(Debug, Error)]\n#[error(\"bad\")]\nstruct Bad { #[source] a: std::io::Error, #[source] b: std::io::Error }",
            "more than one source",
            3,
        ),
        (
            "field",
            "#[derive(Debug, Error)]\n#[error(\"bad\")]\nstruct Bad { #[error(\"field\")] n: u8 }",
            "not allowed on a field",
            3,
        ),
        (
            "placement",
            "#[derive(Debug, Error)]\n#[source]\n#[error(\"bad\")]\nstruct Bad;",
            "only allowed on a field",
            2,
        ),
        (
            "source_args",
            "#[derive(Debug, Error)]\n#[error(\"bad\")]\nstruct Bad(#[source(wrong)] std::io::Error);",
            "unexpected token",
            3,
        ),
        (
            "format",
            "#[derive(Debug, Error)]\n#[error(\"{missing}\")]\nstruct Bad;",
            "missing",
            2,
        ),
        (
            "empty_enum",
            "#[derive(Debug, Error)]\n#[error(12)]\nenum Bad {}",
            "expected a string literal or `transparent`",
            2,
        ),
        (
            "overridden_bad_default",
            "#[derive(Debug, Error)]\n#[error(12)]\nenum Bad { #[error(\"override\")] Unit }",
            "expected a string literal or `transparent`",
            2,
        ),
        (
            "empty_duplicate",
            "#[derive(Debug, Error)]\n#[error(\"one\")]\n#[error(\"two\")]\nenum Bad {}",
            "duplicate #[error]",
            3,
        ),
        (
            "empty_placement",
            "#[derive(Debug, Error)]\n#[from]\nenum Bad {}",
            "only allowed on a field",
            2,
        ),
        (
            "positional_ambiguity",
            "#[derive(Debug, Error)]\n#[error(\"{} {1}\", .0, 7)]\nstruct Bad(u8);",
            "ambiguous positional argument",
            2,
        ),
        (
            "invalid_index",
            "#[derive(Debug, Error)]\n#[error(\"{+0}\")]\nstruct Bad(u8);",
            "invalid format string",
            2,
        ),
        (
            "overflowing_index",
            "#[derive(Debug, Error)]\n#[error(\"{184467440737095516160}\")]\nstruct Bad(u8);",
            "invalid format string",
            2,
        ),
        (
            "option_arity",
            "type Option<A, B> = (A, B);\n#[derive(Debug, Error)]\n#[error(\"bad\")]\nstruct Bad(#[from] Option<std::io::Error, u8>);",
            "as_dyn_error",
            2,
        ),
        (
            "missing_variant",
            "#[derive(Debug, Error)]\nenum Bad {\n    #[error(\"one\")]\n    One,\n    Two,\n}",
            "missing #[error] attribute on variant",
            5,
        ),
    ] {
        let dir = package(&format!("error-negative-{name}"), source);
        let out = driver(&dir)
            .args([
                "--crate-type=lib",
                "--emit=metadata",
                "--error-format=short",
            ])
            .output()
            .unwrap();
        assert_eq!(out.status.code(), Some(1), "{name}: {}", stderr(&out));
        let err = stderr(&out);
        assert!(
            err.lines()
                .any(|l| l.contains(&format!("demo.rs:{line}:")) && l.contains(message)),
            "{name}: {err}"
        );
        assert!(!dir.join("out/demo").exists());
    }
}

#[test]
fn error_derive_formats_the_original_pointer_with_an_unsafe_grant() {
    let dir = package(
        "error-pointer",
        r#"
#[derive(Debug, Error)]
#[error("{address:p}")]
struct Address<'a> { address: &'a u8 }
#[tide::main]
fn main() {
    let byte = 7;
    let address = &byte;
    assert_eq!(Address { address }.to_string(), format!("{address:p}"));
    println!("POINTER_RAN");
}
"#,
    );
    checked(driver(&dir).arg("--tide-grant=unsafe"));
    let out = checked(&mut Command::new(dir.join("out/demo")));
    assert_eq!(out.stdout, b"POINTER_RAN\n");
}

#[test]
fn generic_error_types_still_require_pure() {
    let dir = package(
        "error-generic-pure",
        r#"
#[derive(Debug, Error)]
#[error("bounded")]
enum Bounded<T: Pure> { Value { value: T } }
#[tide::main]
fn main(root: Root) { let _ = Bounded::Value { value: root.net }; }
"#,
    );
    let out = driver(&dir).output().unwrap();
    assert_eq!(out.status.code(), Some(1), "{}", stderr(&out));
    assert!(stderr(&out).contains("TD0517"), "{}", stderr(&out));
    assert!(!dir.join("out/demo").exists());
}

#[test]
fn generic_error_patterns_keep_distinct_type_arguments() {
    let dir = package(
        "error-pattern-args",
        r#"
#[derive(Debug, Error)]
#[error("bounded")]
enum Bounded<P: Pure, T> { Value { marker: P, value: T } }
#[tide::main]
fn main(root: Root) {
    let e = Bounded::Value { marker: 7u8, value: root.net };
    assert_eq!(e.to_string(), "bounded");
    println!("PATTERNS_RAN");
}
"#,
    );
    let out = checked(&mut driver(&dir));
    assert!(out.stderr.is_empty(), "{}", stderr(&out));
    let out = checked(&mut Command::new(dir.join("out/demo")));
    assert_eq!(out.stdout, b"PATTERNS_RAN\n");
}

/// Execute the shared production/conformance file implementation through admission.
#[test]
fn real_files_build_and_run_against_production_std() {
    use std::os::unix::fs::symlink;
    let source = include_str!("../../../conformance/cases/authority/files_executable.rs");
    let dir = package("real-files", source);
    let base = dir.join("files");
    std::fs::create_dir_all(base.join("links")).unwrap();
    std::fs::create_dir_all(base.join("synced")).unwrap();
    std::fs::write(base.join("in.txt"), "safe").unwrap();
    std::fs::write(dir.join("outside"), "untouched").unwrap();
    symlink("../in.txt", base.join("links/ok")).unwrap();
    symlink("../..", base.join("links/up")).unwrap();
    symlink("/etc", base.join("links/abs")).unwrap();
    symlink("../synced", base.join("links/dir")).unwrap();
    symlink("missing-target", base.join("links/dangling")).unwrap();
    symlink("synced/created", base.join("new-link")).unwrap();
    symlink(
        format!("{}synced/long-target", "./".repeat(160)),
        base.join("long-link"),
    )
    .unwrap();
    std::fs::create_dir(base.join("deep")).unwrap();
    std::fs::write(base.join("deep/target"), "unchanged").unwrap();
    symlink("deep", base.join("alias")).unwrap();
    for n in 0..40 {
        let target = if n == 39 {
            "target".to_owned()
        } else {
            format!("l{}", n + 1)
        };
        symlink(target, base.join(format!("deep/l{n}"))).unwrap();
    }
    symlink("l0", base.join("deep/too-many")).unwrap();
    checked(&mut driver(&dir));
    let out = checked(Command::new(dir.join("out/demo")).current_dir(&dir));
    assert_eq!(out.stdout, b"FILES_RAN\n");
    assert!(out.stderr.is_empty(), "{}", stderr(&out));
    assert_eq!(std::fs::read_to_string(dir.join("outside")).unwrap(), "untouched");
}

#[test]
fn simulation_tests_find_lost_updates_and_reexecute_seeds() {
    let dir = package(
        "simulation-runner",
        include_str!("../../../conformance/cases/tasks/simulation_runner.rs"),
    );
    checked(driver(&dir).args(["--crate-type=lib", "--test"]));
    let executable = dir.join("out/demo-test");
    let run = |name: &str, seed: Option<&str>| {
        let mut command = Command::new(&executable);
        command.args(["--exact", name, "--nocapture", "--include-ignored"]);
        command
            .env_remove("TIDE_SIM_SEED")
            .env_remove("__TIDE_SIM_CHILD");
        if let Some(seed) = seed {
            command.env("TIDE_SIM_SEED", seed);
        }
        command.stdin(Stdio::null()).output().unwrap()
    };
    for (name, count) in [
        ("fixed_updates", 20),
        ("default_seeds", 100),
        ("ignored_simulation", 1),
        ("r#loop", 1),
        ("empty_range", 0),
    ] {
        let out = run(&format!("simulation::{name}"), None);
        let output = format!("{}{}", String::from_utf8_lossy(&out.stdout), stderr(&out));
        assert!(out.status.success(), "{output}");
        assert!(output.contains(&format!("{count} passed; 0 failed ({count} seeds)")), "{output}");
        assert!(!output.contains("TIDE_SIM_SEED_START"), "{output}");
        assert_eq!(output.matches("test result:").count(), 1, "{output}");
    }
    let captured = checked(Command::new(&executable).args(["--exact", "simulation::fixed_updates"]));
    let output = String::from_utf8_lossy(&captured.stdout);
    assert_eq!(output.matches("test result:").count(), 1, "{output}");
    assert!(!output.contains("TIDE_SIM_SEED_START"), "{output}");
    assert!(!output.contains("simulation simulation::fixed_updates:"), "{output}");
    assert!(captured.stderr.is_empty(), "{}", stderr(&captured));
    let selected = run("simulation::fixed_updates", Some("19"));
    let output = format!(
        "{}{}",
        String::from_utf8_lossy(&selected.stdout),
        stderr(&selected)
    );
    assert!(selected.status.success(), "{output}");
    assert!(output.contains("1 passed; 0 failed (1 seeds)"), "{output}");
    let invalid = run("simulation::fixed_updates", Some("invalid"));
    assert!(!invalid.status.success());
    assert!(stderr(&invalid).contains("TIDE_SIM_SEED must be a decimal u64"));
    let out = run("simulation::plain_simulation", None);
    assert!(out.status.success(), "{}", stderr(&out));
    assert!(String::from_utf8_lossy(&out.stdout).contains("PLAIN_SIM_RAN"));
    assert!(!String::from_utf8_lossy(&out.stdout).contains("quote="));
    assert!(out.stderr.is_empty(), "{}", stderr(&out));

    let out = run("simulation::loses_updates", None);
    let output = format!("{}{}", String::from_utf8_lossy(&out.stdout), stderr(&out));
    assert!(!out.status.success(), "{output}");
    assert!(output.contains("lost update"), "{output}");
    let failed = output
        .lines()
        .find_map(|line| line.strip_prefix("simulation failure seed="))
        .and_then(|line| line.split_whitespace().next())
        .expect(&output);
    assert!(output.contains("(20 seeds)"), "{output}");
    assert!(output.contains("failing seeds:"), "{output}");
    assert!(!output.contains("last started seed"), "{output}");
    assert!(!output.contains("TIDE_SIM_SEED_START"), "{output}");
    let captured = Command::new(&executable)
        .args(["--exact", "simulation::loses_updates"])
        .env("TIDE_SIM_SEED", failed)
        .env_remove("__TIDE_SIM_CHILD")
        .stdin(Stdio::null()).output().unwrap();
    assert!(!captured.status.success());
    assert!(captured.stderr.is_empty(), "{}", stderr(&captured));
    let captured_output = String::from_utf8_lossy(&captured.stdout);
    assert!(captured_output.contains("lost update"), "{captured_output}");
    assert!(captured_output.contains("\"kind\":\"sink\""), "{captured_output}");
    // One fixed seed in two fresh child processes must give the same trace and sink output.
    let trace = |out: Output| {
        assert!(!out.status.success());
        stderr(&out)
            .lines()
            .filter(|line| line.starts_with('{'))
            .map(str::to_owned)
            .collect::<Vec<_>>()
    };
    let first = trace(run("simulation::loses_updates", Some(failed)));
    assert!(!first.is_empty());
    assert!(first.iter().any(|line| line.contains("\"kind\":\"sched\"")));
    assert!(first.iter().any(|line| line.contains("\"kind\":\"sink\"")));
    assert_eq!(first, trace(run("simulation::loses_updates", Some(failed))));
}

#[test]
fn sim_test_should_panic_passes_only_when_a_seed_panics_as_expected() {
    let dir = package(
        "simulation-should-panic",
        r#"
#[sim_test(seeds = 3)]
#[should_panic]
fn bare(sim: std::sim::SimRoot) { let _ = sim; panic!("gauge overflow"); }
#[sim_test(seeds = 3)]
#[should_panic(expected = "overflow")]
fn matching(sim: std::sim::SimRoot) { let _ = sim; panic!("gauge overflow"); }
#[sim_test(seeds = 3)]
#[should_panic(expected = "underflow")]
fn mismatched(sim: std::sim::SimRoot) { let _ = sim; panic!("gauge overflow"); }
#[sim_test(seeds = 3)]
#[should_panic]
fn quiet(sim: std::sim::SimRoot) { let _ = sim; }
#[sim_test(seeds = 3)]
#[should_panic]
fn returns_err(sim: std::sim::SimRoot) -> Result<(), String> { let _ = sim; Err("no panic".into()) }
"#,
    );
    checked(driver(&dir).args(["--crate-type=lib", "--test"]));
    for (name, passes) in [
        ("bare", true),
        ("matching", true),
        ("mismatched", false),
        ("quiet", false),
        ("returns_err", false),
    ] {
        let out = Command::new(dir.join("out/demo-test"))
            .args(["--exact", name])
            .env_remove("TIDE_SIM_SEED")
            .env_remove("__TIDE_SIM_CHILD")
            .stdin(Stdio::null())
            .output()
            .unwrap();
        let output = format!("{}{}", String::from_utf8_lossy(&out.stdout), stderr(&out));
        assert_eq!(out.status.success(), passes, "{name}: {output}");
        assert!(output.contains(if passes { "1 passed" } else { "1 failed" }), "{name}: {output}");
    }
}

/// The same file program executes on production real and simulated Fs, then compares results.
#[test]
fn simulated_files_match_real_files_and_replay() {
    let dir = package("simulated-files", include_str!("../../../conformance/cases/authority/simulated_files.rs"));
    std::fs::create_dir_all(dir.join("files")).unwrap();
    checked(&mut driver(&dir));
    let real = checked(Command::new(dir.join("out/demo")).current_dir(&dir));
    checked(driver(&dir).args(["--crate-type=lib", "--test"]));
    let test = dir.join("out/demo-test");
    let simulated = checked(Command::new(&test).args(["--exact", "simulated::equivalent_program", "--nocapture"]));
    let observation = |out: &Output| String::from_utf8_lossy(&out.stdout).lines().find(|l| l.starts_with("OBS:")).unwrap().to_owned();
    assert_eq!(observation(&real), observation(&simulated));
    for name in ["fixed_seed_trace", "setup_time_and_cancellation", "cancellation_during_latency_returns_the_operation_result"] {
        let out = checked(Command::new(&test).args(["--exact", &format!("simulated::{name}"), "--nocapture"]));
        assert!(String::from_utf8_lossy(&out.stdout).contains("1 passed; 0 failed"));
    }
}

#[test]
fn trait_dispatch_recursion_keeps_rustcs_failure_without_an_uncertain_label() {
    // The parameter dispatch in bounce cannot prove a cycle back to the blanket impl.
    let dir = package(
        "trait-dispatch-recursion",
        r#"
trait Step { fn step(self); }
impl<T> Step for T { fn step(self) { bounce(Some(self)); } }
fn bounce<T: Step>(x: T) { x.step(); }
#[tide::main]
fn main(_: Root) { bounce(0u8); }
"#,
    );
    let output = driver(&dir).output().unwrap();
    let error = stderr(&output);
    assert_eq!(output.status.code(), Some(1), "{error}");
    assert!(
        error.contains("reached the recursion limit while instantiating"),
        "{error}"
    );
    assert!(!error.contains("TD0407"), "{error}");
    assert!(!dir.join("out/demo").exists());
}

#[test]
fn synthesized_impls_are_trusted_without_trusting_adjacent_user_impls() {
    let source = r#"
mod generated {
    #[derive(Debug)] struct Bag<T> { rows: Vec<T> }
    #[test] fn check(value: Bag<u8>) { let _ = value; }
}
"#;
    let dir = package("trait-synthesis-provenance-accept", source);
    checked(driver(&dir).args(["--crate-type=lib", "--test"]));
    let dir = package(
        "trait-synthesis-provenance-reject",
        &format!(
            "{source}\npub trait Marker {{}}\nimpl<T> Marker for T where Vec<T>: Clone {{}}\n"
        ),
    );
    let output = driver(&dir)
        .args(["--crate-type=lib", "--test"])
        .output()
        .unwrap();
    let error = stderr(&output);
    assert_eq!(output.status.code(), Some(1), "{error}");
    let labels: Vec<_> = error
        .lines()
        .filter(|line| line.contains("error: TD"))
        .collect();
    assert_eq!(labels.len(), 1, "{error}");
    assert!(
        labels[0].contains("demo.rs:8:") && labels[0].contains("TD0406:"),
        "{error}"
    );
}

#[test]
fn finite_trait_dispatch_chain_keeps_rustcs_limit_failure_without_a_growth_label() {
    let mut source = String::from("pub trait K { fn step(); }\nfn f<T: K>() { T::step(); }\n");
    for n in 0..9 {
        let body = if n == 0 {
            String::new()
        } else {
            format!("f::<S{}>();", n - 1)
        };
        source.push_str(&format!(
            "pub struct S{n}; impl K for S{n} {{ fn step() {{ {body} }} }}\n"
        ));
    }
    source.push_str("pub fn entry() { f::<S8>(); }\n");
    for (limit, success) in [(4, false), (128, true)] {
        let dir = package(
            &format!("finite-trait-chain-{limit}"),
            &format!("#![recursion_limit=\"{limit}\"]\n{source}"),
        );
        let output = driver(&dir)
            .args(["--crate-type=lib", "--emit=metadata"])
            .output()
            .unwrap();
        let error = stderr(&output);
        assert_eq!(output.status.success(), success, "{error}");
        assert!(!error.contains("TD0407"), "{error}");
        if !success {
            assert!(
                error.contains("reached the recursion limit while instantiating"),
                "{error}"
            );
        }
    }
}

#[test]
fn arena_and_pure_build_and_execute_against_production_std() {
    let dir = package(
        "arena-executable",
        include_str!("../../../conformance/cases/memory/arena_executable.rs"),
    );
    checked(&mut driver(&dir));
    let out = checked(&mut Command::new(dir.join("out/demo")));
    assert_eq!(out.stdout, b"ARENA_RAN\n");
    assert!(out.stderr.is_empty(), "{}", stderr(&out));

    for access in ["g[stale]", "{ g[stale] = 9; 0 }"] {
        let dir = package(
            "arena-stale",
            &format!(
                r#"
use std::arena::Arena;
#[tide::main]
fn main() {{
    let mut g = Arena::new();
    let stale = g.insert(7);
    assert_eq!(g.remove(stale), Some(7));
    g.insert(8);
    assert!(g.get(stale).is_none());
    let _ = {access};
}}
"#
            ),
        );
        checked(&mut driver(&dir));
        let out = Command::new(dir.join("out/demo"))
            .stdin(Stdio::null())
            .output()
            .unwrap();
        assert_eq!(out.status.code(), Some(101), "{}", stderr(&out));
        assert!(stderr(&out).contains("stale arena id"), "{}", stderr(&out));
    }
}

#[test]
fn arena_and_pure_rejections_use_the_existing_driver_rules() {
    for (name, source, codes) in [
        (
            "arena",
            include_str!("../../../conformance/cases/memory/arena_pure_reject.rs"),
            &["TD0517", "TD0511"][..],
        ),
        (
            "capability",
            include_str!("../../../conformance/cases/authority/afree_pure_capability.rs"),
            &["TD0511"][..],
        ),
        (
            "generic",
            include_str!("../../../conformance/cases/authority/afree_pure_generic.rs"),
            &["TD0511"][..],
        ),
        (
            "nested",
            include_str!("../../../conformance/cases/authority/afree_pure_nested.rs"),
            &["TD0511"][..],
        ),
    ] {
        let dir = package(&format!("arena-pure-{name}"), source);
        let out = driver(&dir)
            .args(["--crate-type=lib", "--emit=metadata"])
            .stdin(Stdio::null())
            .output()
            .unwrap();
        assert_eq!(out.status.code(), Some(1), "{name}: {}", stderr(&out));
        for code in codes {
            assert!(stderr(&out).contains(code), "{name}: {}", stderr(&out));
        }
        assert!(!dir.join("out/demo").exists());
    }
    let dir = package(
        "arena-pure-accept",
        include_str!("../../../conformance/cases/authority/afree_pure_accept.rs"),
    );
    checked(driver(&dir).args(["--crate-type=lib", "--emit=metadata"]));
}

#[test]
fn pure_attribute_rejects_arguments_and_non_functions() {
    for (source, message) in [
        ("#[pure(wrong)] fn f() {}", "`#[pure]` takes no arguments"),
        ("#[pure] struct Bad;", "`#[pure]` goes on a function"),
        (
            "#[pure] mod bad { fn f() {} }",
            "`#[pure]` goes on a function",
        ),
        (
            "#[pure] const BAD: fn() = || {};",
            "`#[pure]` goes on a function",
        ),
    ] {
        let dir = package("pure-attribute-invalid", source);
        let out = driver(&dir)
            .args(["--crate-type=lib", "--emit=metadata"])
            .stdin(Stdio::null())
            .output()
            .unwrap();
        assert_eq!(out.status.code(), Some(1), "{}", stderr(&out));
        assert!(stderr(&out).contains(message), "{}", stderr(&out));
        assert!(stderr(&out).contains("demo.rs:1:"), "{}", stderr(&out));
    }
}

/// TCP's one equivalence suite executes through tide-admit, including borrowed I/O and cancellation.
#[test]
fn tcp_builds_and_runs_against_production_std() {
    let dir = package(
        "tcp",
        include_str!("../../../conformance/cases/authority/tcp.rs"),
    );
    for profile in ["debug", "release"] {
        checked(driver(&dir).arg(format!("--tide-profile={profile}")));
        let out = checked(
            Command::new("timeout")
                .args(["45"])
                .arg(dir.join("out/demo")),
        );
        let real = String::from_utf8(out.stdout.clone()).unwrap();
        let real = real.strip_prefix("TCP_VALUES:").unwrap();
        let real = real.strip_suffix("\nTCP_RAN\n").unwrap();
        // Isolated unwinds are contained; their host panic hooks still run.
        let hooks = stderr(&out);
        assert_eq!(hooks.matches("tcp listener unwind").count(), 1, "{hooks}");
        assert_eq!(hooks.matches("tcp stream unwind").count(), 1, "{hooks}");
        assert_eq!(hooks.matches("tcp rebind unwind").count(), 0, "{hooks}");
        checked(driver(&dir).args(["--test", &format!("--tide-profile={profile}")]));
        let out = checked(
            Command::new("timeout")
                .args(["45"])
                .arg(dir.join("out/demo-test"))
                .args(["--exact", "simulated_tcp", "--nocapture"]),
        );
        let simulated = String::from_utf8_lossy(&out.stdout);
        assert!(simulated.contains("1 passed"));
        let values: Vec<_> = simulated
            .split("TCP_VALUES:")
            .skip(1)
            .map(|text| text.lines().next().unwrap())
            .collect();
        assert_eq!(values, vec![real; 4], "{simulated}");
        let hooks = stderr(&out);
        assert_eq!(hooks.matches("tcp listener unwind").count(), 10, "{hooks}");
        assert_eq!(hooks.matches("tcp stream unwind").count(), 10, "{hooks}");
        assert_eq!(hooks.matches("tcp rebind unwind").count(), 5, "{hooks}");
    }
}

#[test]
fn tcp_backpressure_deadlock_names_the_connection() {
    let dir = package(
        "tcp-deadlock",
        include_str!("../../../conformance/cases/authority/tcp_deadlock.rs"),
    );
    checked(driver(&dir).args(["--crate-type=lib", "--test"]));
    let out = Command::new("timeout")
        .arg("15")
        .arg(dir.join("out/demo-test"))
        .args(["--exact", "deadlock", "--nocapture"])
        .stdin(Stdio::null())
        .output()
        .unwrap();
    assert!(!out.status.success());
    assert_ne!(
        out.status.code(),
        Some(124),
        "TCP wait stalled instead of reporting deadlock"
    );
    assert!(
        stderr(&out).contains("net.write(conn 0)"),
        "{}",
        stderr(&out)
    );
    assert!(
        stderr(&out).contains("tide-source/demo.rs"),
        "{}",
        stderr(&out)
    );
}

#[test]
fn tcp_has_no_ambient_constructors_or_resolution() {
    for (name, source, diagnostic) in [
        (
            "listener",
            "fn run() { let _ = std::net::TcpListener::bind(\"127.0.0.1:0\"); }",
            "E0599",
        ),
        (
            "stream",
            "fn run() { let _ = std::net::TcpStream::connect(\"127.0.0.1:1\"); }",
            "E0599",
        ),
        (
            "resolve",
            "fn run(a: impl std::net::ToSocketAddrs) { let _ = a.address().unwrap().resolve(false); }",
            "E0624",
        ),
    ] {
        let dir = package(&format!("tcp-no-ambient-{name}"), source);
        let out = driver(&dir)
            .args(["--crate-type=lib", "--emit=metadata"])
            .stdin(Stdio::null())
            .output()
            .unwrap();
        assert!(!out.status.success());
        assert!(stderr(&out).contains(diagnostic), "{}", stderr(&out));
    }
}

#[test]
fn tcp_incoming_carries_network_authority() {
    for (name, source, code) in [
        (
            "pure",
            "fn pure_value(_: impl Pure) {}\npub fn run(i: std::net::Incoming<'_>) { pure_value(i); }",
            "TD0517",
        ),
        (
            "shared",
            "pub fn run(i: std::net::Incoming<'_>, tasks: &std::task::Tasks) { let _ = std::sync::Shared::new(i, tasks); }",
            "TD0606",
        ),
        (
            "pure-function",
            "#[pure]\npub fn run(mut i: std::net::Incoming<'_>) { let _ = i.next(); }",
            "TD0511",
        ),
    ] {
        let dir = package(&format!("tcp-incoming-{name}"), source);
        let out = driver(&dir)
            .args(["--crate-type=lib", "--emit=metadata"])
            .stdin(Stdio::null())
            .output()
            .unwrap();
        assert!(!out.status.success(), "Incoming admitted as Pure: {name}");
        assert!(stderr(&out).contains(code), "{}", stderr(&out));
        assert!(stderr(&out).contains("Incoming"), "{}", stderr(&out));
    }
    let dir = package(
        "tcp-incoming-authority",
        "pub fn incoming(_: std::net::Incoming<'_>) {}",
    );
    checked(driver(&dir).args(["--crate-type=lib", "--emit=metadata"]));
    let artifact = std::fs::read_to_string(dir.join("out/demo.interface.json")).unwrap();
    assert!(
        artifact.contains(r##""authority": {"#0": ["net"]}"##),
        "{artifact}"
    );
}

#[test]
fn authority_rules_reject_at_error_severity_against_production_std() {
    for (name, source, code) in [
        (
            "ambient",
            "pub fn inspect() { let _ = std::time::Instant::now(); }",
            "TD0501",
        ),
        (
            "root",
            "#[tide::main] fn main(root: Root) { let _all = root; }",
            "TD0503",
        ),
        (
            "simulation",
            "pub fn setup() { let _ = Root::sim(1); }",
            "TD0504",
        ),
        (
            "handle",
            "pub fn setup() { let _ = std::sync::Shared::new(1u64); }",
            "TD0505",
        ),
    ] {
        let dir = package(&format!("authority-{name}"), source);
        for test in [false, true] {
            let mut command = driver(&dir);
            command.args([
                "--crate-type=lib",
                "--emit=metadata",
                "--error-format=short",
            ]);
            if test {
                command.arg("--test");
            }
            let out = command.output().unwrap();
            assert_eq!(out.status.code(), Some(1), "{}", stderr(&out));
            assert!(
                stderr(&out).contains(&format!("error: {code}:")),
                "{}",
                stderr(&out)
            );
            assert!(
                !stderr(&out).contains(&format!("warning: {code}:")),
                "{}",
                stderr(&out)
            );
        }
    }
}

#[test]
fn authority_simulation_is_available_to_separate_suites_but_not_the_candidate() {
    let dir = package(
        "authority-paired-simulation",
        "fn setup() { let _ = Root::sim(1); }\n#[test] fn uses_it() { setup(); }",
    );
    let candidate = dir.join("candidate/src/lib.rs");
    std::fs::create_dir_all(candidate.parent().unwrap()).unwrap();
    std::fs::write(
        &candidate,
        "pub fn value() -> u64 { 1 } pub fn thread_rng() -> u64 { 4 }",
    )
    .unwrap();
    checked(paired_test_driver(&dir).arg("--emit=metadata"));
    std::fs::write(
        dir.join("package/src/demo.rs"),
        "#[test] fn draw() { assert_eq!(tide_candidate::thread_rng(), 4); }",
    )
    .unwrap();
    checked(paired_test_driver(&dir).arg("--emit=metadata"));
    std::fs::write(&candidate, "pub fn setup() { let _ = Root::sim(1); }").unwrap();
    let out = paired_test_driver(&dir)
        .arg("--emit=metadata")
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1), "{}", stderr(&out));
    assert!(stderr(&out).contains("TD0504"), "{}", stderr(&out));
}

#[test]
fn authority_field_patterns_and_import_sources_use_production_std() {
    for (name, source, kind, rejected) in [
        (
            "root-destructure",
            include_str!("../../../conformance/cases/authority/root_destructure_accept.rs"),
            "bin",
            None,
        ),
        (
            "root-parameter-pattern",
            include_str!("../../../conformance/cases/authority/root_parameter_pattern_accept.rs"),
            "bin",
            None,
        ),
        (
            "root-helper-pattern",
            include_str!("../../../conformance/cases/authority/root_parameter_pattern_reject.rs"),
            "lib",
            Some("TD0503"),
        ),
        (
            "local-rng",
            include_str!("../../../conformance/cases/authority/cap_ambient_accept.rs"),
            "lib",
            None,
        ),
        (
            "aliased-denial",
            include_str!("../../../conformance/cases/authority/admit_std_alias_reject.rs"),
            "lib",
            Some("TD0520"),
        ),
        (
            "thread-timers",
            include_str!("../../../conformance/cases/authority/cap_thread_timers_reject.rs"),
            "lib",
            Some("TD0501"),
        ),
        (
            "root-match-binding",
            include_str!("../../../conformance/cases/authority/root_match_binding_reject.rs"),
            "bin",
            Some("TD0503"),
        ),
        (
            "root-pattern-capture",
            include_str!("../../../conformance/cases/authority/root_pattern_capture_accept.rs"),
            "bin",
            None,
        ),
        (
            "local-command",
            include_str!("../../../conformance/cases/authority/cap_ambient_import_accept.rs"),
            "lib",
            None,
        ),
        (
            "ambient-command",
            include_str!("../../../conformance/cases/authority/cap_ambient_import_reject.rs"),
            "lib",
            Some("TD0501"),
        ),
    ] {
        let dir = package(&format!("authority-{name}"), source);
        for test in [false, true] {
            let mut command = driver(&dir);
            command.args([&format!("--crate-type={kind}"), "--emit=metadata"]);
            if test {
                command.arg("--test");
            }
            let out = command.output().unwrap();
            assert_eq!(
                out.status.code(),
                Some(if rejected.is_some() { 1 } else { 0 }),
                "{}",
                stderr(&out)
            );
            if let Some(code) = rejected {
                assert!(stderr(&out).contains(code), "{}", stderr(&out));
            }
        }
    }
}

#[test]
fn concurrency_and_error_cases_use_production_std() {
    for (name, source, rejection) in [
        (
            "async",
            include_str!("../../../conformance/cases/tasks/async_reject.rs"),
            Some("TD0601"),
        ),
        (
            "future-pin",
            include_str!("../../../conformance/cases/tasks/future_pin_reject.rs"),
            Some("TD0601"),
        ),
        (
            "spawn",
            include_str!("../../../conformance/cases/tasks/spawn_reject.rs"),
            Some("TD0602"),
        ),
        (
            "mutex",
            include_str!("../../../conformance/cases/tasks/mutex_reject.rs"),
            Some("TD0604"),
        ),
        (
            "select",
            include_str!("../../../conformance/cases/tasks/select_reject.rs"),
            Some("TD0605"),
        ),
        (
            "catch",
            include_str!("../../../conformance/cases/errors/catch_unwind_reject.rs"),
            Some("TD0701"),
        ),
        (
            "result",
            include_str!("../../../conformance/cases/errors/ignored_result_reject.rs"),
            Some("TD0702"),
        ),
        (
            "ordinary",
            include_str!("../../../conformance/cases/tasks/async_accept.rs"),
            None,
        ),
        (
            "scoped",
            include_str!("../../../conformance/cases/tasks/spawn_accept.rs"),
            None,
        ),
        (
            "shared",
            include_str!("../../../conformance/cases/tasks/mutex_accept.rs"),
            None,
        ),
        (
            "channel",
            include_str!("../../../conformance/cases/tasks/select_accept.rs"),
            None,
        ),
        (
            "isolated",
            include_str!("../../../conformance/cases/errors/catch_unwind_accept.rs"),
            None,
        ),
        (
            "handled",
            include_str!("../../../conformance/cases/errors/ignored_result_accept.rs"),
            None,
        ),
    ] {
        let dir = package(&format!("habits-{name}"), source);
        for test in [false, true] {
            let mut command = driver(&dir);
            command.args([
                "--crate-type=lib",
                "--emit=metadata",
                "--error-format=short",
            ]);
            if test {
                command.arg("--test");
            }
            let out = command.stdin(Stdio::null()).output().unwrap();
            assert_eq!(
                out.status.code(),
                Some(if rejection.is_some() { 1 } else { 0 }),
                "{}",
                stderr(&out)
            );
            if let Some(code) = rejection {
                assert!(
                    stderr(&out).contains(&format!("error: {code}:")),
                    "{}",
                    stderr(&out)
                );
            }
        }
    }
}


// Use the production reader to edit fixtures without a second JSON implementation.
use std::collections;
#[allow(dead_code)]
#[path = "../../tide-std/src/codec.rs"]
mod __codec;
#[allow(dead_code)]
#[path = "../../tide-std/src/json.rs"]
mod json;
#[allow(dead_code)]
#[path = "../../tide-std/src/replay.rs"]
mod replay_wire;

#[test]
fn ordinary_const_tests_keep_direct_and_const_context_calls() {
    let dir = package("ordinary-const-calls", r#"
#[test]
const fn check() { assert!(2 + 2 == 4); }
#[test]
fn composed() {
    check();
    const _: () = check();
}
"#);
    checked(driver(&dir).args(["--crate-type=lib", "--test"]));
    let out = checked(Command::new(dir.join("out/demo-test")).args(["--test-threads=4"]));
    assert!(String::from_utf8_lossy(&out.stdout).contains("2 passed"));
}

#[test]
fn ordinary_result_tests_keep_their_signature_and_direct_calls() {
    let dir = package("ordinary-result-calls", r#"
#[test]
fn check() -> Result<(), String> { Ok(()) }
#[test]
#[ignore]
fn failed() -> Result<(), String> { Err("returned error".into()) }
struct Status;
impl std::process::Termination for Status {
    fn report(self) -> std::process::ExitCode { std::process::ExitCode::SUCCESS }
}
#[test]
fn custom() -> Status { Status }
#[test]
fn composed() {
    let callable: fn() -> Result<(), String> = check;
    callable().unwrap();
    check().unwrap();
    assert_eq!(failed().unwrap_err(), "returned error");
    let _: Status = custom();
}
"#);
    checked(driver(&dir).args(["--crate-type=lib", "--test"]));
    checked(Command::new(dir.join("out/demo-test")).args(["--test-threads=4"]));
    let failed = Command::new(dir.join("out/demo-test")).args(["--exact", "failed", "--ignored", "--nocapture", "--test-threads=4"])
        .stdin(Stdio::null()).output().unwrap();
    assert!(!failed.status.success());
    assert!(stderr(&failed).contains("returned error"), "{}", stderr(&failed));
}

#[test]
fn ordinary_unit_tests_compose_in_one_recorded_and_replayed_evaluation() {
    use replay_wire::{Manifest, read};
    let dir = package("ordinary-unit-calls", r#"
#[test]
fn check() {
    let sim = Root::sim(17);
    println!("nested call");
    sim.tasks.checkpoint().unwrap();
}
#[test]
fn leaf() { println!("child call"); }
#[test]
fn composed() {
    check(); check();
    let sim = Root::sim(29);
    sim.tasks.scope::<_, std::task::TaskError, _>(|scope| {
        scope.spawn(|| { leaf(); Ok(()) });
        Ok(())
    }).unwrap();
}
"#);
    checked(driver(&dir).args(["--crate-type=lib", "--test"]));
    let direct = Command::new(dir.join("out/demo-test")).args(["--exact", "composed", "--test-threads=4"])
        .stdin(Stdio::null()).output().unwrap();
    assert!(direct.status.success(), "{}{}", String::from_utf8_lossy(&direct.stdout), stderr(&direct));
    let manifests = || std::fs::read_dir(dir.join("out/target/sim")).unwrap()
        .map(|e| e.unwrap().path().join("run.manifest.json")).collect::<Vec<_>>();
    for (name, seeds) in [("check", vec!["17"]), ("composed", vec!["17", "17", "29"])] {
        let before = if dir.join("out/target/sim").is_dir() { manifests() } else { vec![] };
        checked(driver(&dir).args(["--crate-type=lib", "--test", "--tide-run-test", name, "--seed", "47"]));
        let added = manifests().into_iter().filter(|p| !before.contains(p)).collect::<Vec<_>>();
        assert_eq!(added.len(), 1, "one manifest per harness evaluation");
        let recorded: Manifest = read(&added[0]).unwrap();
        recorded.validate().unwrap();
        assert_eq!(recorded.test, name);
        assert!(recorded.complete);
        assert_eq!(recorded.outcome, "returned");
        assert_eq!(recorded.roots.iter().map(|r| r.seed.as_str()).collect::<Vec<_>>(), seeds);
        assert_eq!(recorded.trace.matches("\"kind\":\"sink\"").count(), seeds.len());
        for _ in 0..2 {
            let before = manifests();
            let run = checked(driver(&dir).args(["--crate-type=lib", "--test", "--replay"]).arg(&added[0]));
            assert!(stderr(&run).contains("replay reproduced"), "{}", stderr(&run));
            let added = manifests().into_iter().filter(|p| !before.contains(p)).collect::<Vec<_>>();
            assert_eq!(added.len(), 1);
            let replayed: Manifest = read(&added[0]).unwrap();
            replayed.validate().unwrap();
            assert_eq!(replayed.roots, recorded.roots);
            assert_eq!(replayed.trace.as_bytes(), recorded.trace.as_bytes());
            assert_eq!(replayed.outcome, recorded.outcome);
        }
    }
}

fn nested_evaluation_stays_in_its_caller(name: &str, caller: &str, seeds: &[&str]) {
    use replay_wire::{Manifest, read};
    let source = format!(r#"
#[test]
fn property(value: u8) {{ assert!(u32::from(value) <= 255); println!("{{value}}"); }}
#[sim_test(seeds = 1)]
fn simulation(sim: std::sim::SimRoot, value: bool) {{
    let _ = value;
    sim.tasks.checkpoint().unwrap();
}}
{caller}
"#);
    let dir = package(name, &source);
    checked(driver(&dir).args(["--crate-type=lib", "--test"]));
    checked(Command::new(dir.join("out/demo-test")).args(["--exact", "caller", "--test-threads=4"]));
    // Seed-start records stay internal: a nested simulation reports none of its seeds, so a
    // sim-child caller's parent tracks only the caller's own: seed 0 of `seeds = 1` and its
    // determinism re-run.
    let starts = |output: &Output| [&output.stdout, &output.stderr].iter()
        .flat_map(|bytes| String::from_utf8_lossy(bytes).lines()
            .filter(|line| line.starts_with("TIDE_SIM_SEED_START")).map(str::to_owned).collect::<Vec<_>>())
        .collect::<Vec<_>>();
    let child = checked(Command::new(dir.join("out/demo-test"))
        .args(["--exact", "caller", "--nocapture"]).env("__TIDE_SIM_CHILD", "demo::caller"));
    let own: &[&str] = if caller.contains("#[sim_test") { &["TIDE_SIM_SEED_START 0"; 2] } else { &[] };
    assert_eq!(starts(&child), own, "{}", String::from_utf8_lossy(&child.stdout));
    let manifests = || std::fs::read_dir(dir.join("out/target/sim")).unwrap()
        .map(|e| e.unwrap().path().join("run.manifest.json")).collect::<Vec<_>>();
    let run = checked(driver(&dir).args(["--crate-type=lib", "--test", "--tide-run-test", "caller", "--seed", "47"]));
    assert_eq!(starts(&run), Vec::<String>::new(), "{}", String::from_utf8_lossy(&run.stdout));
    let paths = manifests();
    assert_eq!(paths.len(), 1, "one manifest per harness evaluation: {paths:?}");
    assert!(!stderr(&run).contains("  replay: "), "{}", stderr(&run));
    let recorded: Manifest = read(&paths[0]).unwrap();
    recorded.validate().unwrap();
    assert_eq!(recorded.test, "caller");
    // Nested generators and simulator roots are not part of the outer input tape.
    assert!(recorded.choices.is_empty());
    assert_eq!(recorded.roots.iter().map(|r| r.seed.as_str()).collect::<Vec<_>>(), seeds);
    for _ in 0..2 {
        let before = manifests();
        checked(driver(&dir).args(["--crate-type=lib", "--test", "--replay"]).arg(&paths[0]));
        let added = manifests().into_iter().filter(|p| !before.contains(p)).collect::<Vec<_>>();
        assert_eq!(added.len(), 1);
        let replayed: Manifest = read(&added[0]).unwrap();
        replayed.validate().unwrap();
        assert_eq!(replayed.roots, recorded.roots);
        assert_eq!(replayed.trace, recorded.trace);
        assert_eq!(replayed.outcome, recorded.outcome);
    }
}

#[test]
fn replay_nested_property_from_test_uses_callers_evaluation() {
    nested_evaluation_stays_in_its_caller("nested-property", "#[test] fn caller() { property(); let sim = Root::sim(31); sim.tasks.checkpoint().unwrap(); }", &["31"]);
}

#[test]
fn replay_nested_property_from_simulated_task_uses_callers_evaluation() {
    nested_evaluation_stays_in_its_caller("nested-task-property", r#"
#[test] fn caller() {
    let sim = Root::sim(29);
    sim.tasks.scope::<_, std::task::TaskError, _>(|scope| {
        scope.spawn(|| { property(); Ok(()) });
        Ok(())
    }).unwrap();
}
"#, &["29"]);
}

#[test]
fn replay_nested_sim_from_test_uses_callers_evaluation() {
    nested_evaluation_stays_in_its_caller("nested-simulation", "#[test] fn caller() { simulation(); let sim = Root::sim(31); sim.tasks.checkpoint().unwrap(); }", &["31"]);
}

#[test]
fn replay_nested_sim_from_simulated_task_uses_callers_evaluation() {
    nested_evaluation_stays_in_its_caller("nested-task-simulation", r#"
#[test] fn caller() {
    let sim = Root::sim(29);
    let panic = sim.tasks.spawn_isolated(|| simulation()).unwrap_err();
    // A nested Root::sim remains forbidden; its wrapper must not launch or record a child.
    assert!(panic.message.contains("a task runtime is already active"));
}
"#, &["29"]);
}

#[test]
fn replay_nested_property_from_sim_child_uses_callers_evaluation() {
    nested_evaluation_stays_in_its_caller("nested-child-property", r#"
#[sim_test(seeds = 1)] fn caller(sim: std::sim::SimRoot) { drop(sim); property(); }
"#, &["47"]);
}

#[test]
fn replay_nested_sim_from_sim_child_uses_callers_evaluation() {
    nested_evaluation_stays_in_its_caller("nested-child-simulation", r#"
#[sim_test(seeds = 1)] fn caller(sim: std::sim::SimRoot) { drop(sim); simulation(); }
"#, &["47"]);
}

#[test]
fn replay_hints_only_accompany_failures() {
    let dir = package("replay-hints", r#"
#[test] fn plain() {}
#[test]
#[requires(false)]
fn rejected(value: u16) { let _ = value; }
"#);
    for name in ["plain", "rejected"] {
        let out = driver(&dir).args(["--crate-type=lib", "--test", "--tide-run-test", name, "--seed", "0"])
            .stdin(Stdio::null()).output().unwrap();
        assert!(!stderr(&out).contains("  replay: "), "{}", stderr(&out));
        if name == "rejected" {
            assert!(!out.status.success());
            assert!(stderr(&out).contains("vacuous"), "{}", stderr(&out));
        } else {
            assert!(out.status.success(), "{}", stderr(&out));
        }
    }
}

#[test]
fn replay_refusal_preserves_existing_outputs() {
    use replay_wire::{Manifest, Receipt, read, encoded};
    for paired in [false, true] {
        let dir = package(&format!("replay-refusal-outputs-{paired}"), "#[test] fn check() {}");
        if paired {
            std::fs::create_dir_all(dir.join("candidate/src")).unwrap();
            std::fs::write(dir.join("candidate/src/lib.rs"), "pub fn value() -> u8 { 1 }").unwrap();
        }
        let invoke = || {
            if paired { paired_test_driver(&dir) } else {
                let mut command = driver(&dir);
                command.args(["--crate-type=lib", "--test"]);
                command
            }
        };
        checked(invoke().args(["--tide-run-test", "check", "--seed", "1"]));
        let path = std::fs::read_dir(dir.join("out/target/sim")).unwrap().next().unwrap().unwrap().path().join("run.manifest.json");
        let manifest: Manifest = read(&path).unwrap();
        let paths = ["out/demo-test", "out/demo-test.build.json", "out/demo.interface.json"];
        if paired { std::fs::write(dir.join(paths[2]), b"previously published interface").unwrap(); }
        let before = paths.map(|p| std::fs::read(dir.join(p)).unwrap());
        let unchanged = || {
            for (path, bytes) in paths.iter().zip(&before) {
                assert_eq!(&std::fs::read(dir.join(path)).unwrap(), bytes, "{path}");
            }
        };
        for component in ["compiler", "configuration"] {
            let mut bad = manifest.clone();
            if component == "compiler" { bad.build.compiler.push_str("changed"); }
            else { bad.config.insert("scheduler".into(), "changed".into()); }
            bad.seal();
            let bad_path = dir.join("bad.json");
            std::fs::write(&bad_path, encoded(&bad)).unwrap();
            let refused = invoke().arg("--replay").arg(bad_path).stdin(Stdio::null()).output().unwrap();
            assert_eq!(refused.status.code(), Some(2), "{}", stderr(&refused));
            assert!(stderr(&refused).contains(component), "{}", stderr(&refused));
            unchanged();
        }
        if !paired {
            let entry = std::fs::read_dir(dir.join("out/.tide-cache")).unwrap()
                .map(|e| e.unwrap().path()).find(|p| read::<Receipt>(&p.join("build.json"))
                    .is_ok_and(|r| r.build == manifest.build)).unwrap();
            let executable = entry.join("executable");
            let mut corrupt = std::fs::read(&executable).unwrap();
            corrupt.push(0);
            std::fs::write(executable, corrupt).unwrap();
            let refused = invoke().arg("--replay").arg(path).stdin(Stdio::null()).output().unwrap();
            assert_eq!(refused.status.code(), Some(2), "{}", stderr(&refused));
            assert!(stderr(&refused).contains("executable/receipt mismatch"), "{}", stderr(&refused));
            unchanged();
        }
    }
}

#[test]
fn replay_remapping_keeps_diagnostics_and_non_test_locations_real() {
    // Panic Locations are embedded; Config.call comes from proc-macro Span::file.
    let source = r#"
#[tide::main]
fn main() { panic!("real location"); }
#[sim_test(seeds = 1)]
fn check(sim: std::sim::SimRoot) { sim.tasks.checkpoint().unwrap(); panic!("test location"); }
"#;
    let dir = package("replay-remapping", source);
    checked(&mut driver(&dir));
    let out = Command::new(dir.join("out/demo")).stdin(Stdio::null()).output().unwrap();
    let real = dir.join("package/src/demo.rs").display().to_string();
    assert!(stderr(&out).contains(&format!("{real}:3:")), "{}", stderr(&out));
    let run = driver(&dir).args(["--crate-type=lib", "--test", "--tide-run-test", "check", "--seed", "1"])
        .stdin(Stdio::null()).output().unwrap();
    assert!(!run.status.success());
    assert!(stderr(&run).contains("tide-source/demo.rs:5:"), "{}", stderr(&run));
    let path = std::fs::read_dir(dir.join("out/target/sim")).unwrap().next().unwrap().unwrap().path().join("run.manifest.json");
    let manifest: replay_wire::Manifest = replay_wire::read(&path).unwrap();
    let exit = manifest.trace.lines().find(|l| l.contains("\"kind\":\"exit\"") && l.contains("\"task\":0")).unwrap();
    assert!(exit.contains("\"file\":\"tide-source/demo.rs\",\"line\":5"), "{exit}");
    std::fs::write(dir.join("package/src/demo.rs"), "#[tide::main] fn main() { let x: u32 = \"s\"; let _ = x; }").unwrap();
    for args in [vec!["--emit=metadata"], vec!["--emit=link"], vec!["--emit=link", "--test"]] {
        let out = driver(&dir).args(args).stdin(Stdio::null()).output().unwrap();
        assert!(!out.status.success());
        assert!(stderr(&out).contains(&format!("{real}:1:")), "{}", stderr(&out));
        assert!(!stderr(&out).contains("tide-source"), "{}", stderr(&out));
    }
}

#[test]
fn replay_manifests_validate_identity_and_reproduce_exact_evaluations() {
    use replay_wire::{Manifest, encoded, read};
    let original_source = include_str!("../../../conformance/cases/tasks/simulation_runner.rs");
    let dir = package("replay ' quoted", original_source);
    let output = |out: &Output| format!("{}{}", String::from_utf8_lossy(&out.stdout), stderr(out));
    let invoke = |manifest: &Path| {
        driver(&dir).args(["--crate-type=lib", "--test", "--replay"]).arg(manifest)
            .stdin(Stdio::null()).output().unwrap()
    };
    let fresh = driver(&dir).args(["--crate-type=lib", "--test", "--tide-run-test", "simulation::loses_updates"])
        .stdin(Stdio::null()).output().unwrap();
    assert!(!fresh.status.success(), "{}", output(&fresh));
    assert!(output(&fresh).contains("simulation failure seed="), "{}", output(&fresh));
    let printed = output(&fresh).lines().find_map(|line| line.strip_prefix("  replay: ")).expect("printed replay command").to_string();
    // The operator paths contain both spaces and a quote: execute the actual printed command.
    let mut replays = Vec::new();
    let manifests = || {
        std::fs::read_dir(dir.join("out/target/sim")).unwrap().filter_map(|entry| {
            let path = entry.unwrap().path().join("run.manifest.json");
            path.is_file().then_some(path)
        }).collect::<Vec<_>>()
    };
    let recorded = manifests().into_iter().find(|path| {
        let manifest: Manifest = read(path).unwrap();
        manifest.complete && manifest.outcome.starts_with("panic:")
    }).expect("complete failure manifest");
    let recording: Manifest = read(&recorded).unwrap();
    recording.validate().unwrap();
    assert!(recording.trace.contains("\"kind\":\"sched\""));
    assert!(recording.trace.contains("\"kind\":\"sink\""));
    let task_zero_exit = recording.trace.lines().find(|line| line.contains("\"task\":0") && line.contains("\"kind\":\"exit\"")).unwrap();
    assert!(task_zero_exit.contains("tide-source/demo.rs"), "{task_zero_exit}");
    assert!(!task_zero_exit.contains("runner.rs"));
    let before_printed = manifests();
    let printed_output = Command::new("bash").args(["-c", &printed]).stdin(Stdio::null()).output().unwrap();
    assert!(!printed_output.status.success());
    assert!(output(&printed_output).contains("replay reproduced"), "{}", output(&printed_output));
    assert!(manifests().len() > before_printed.len());
    for _ in 0..2 {
        let before = manifests();
        let run = invoke(&recorded);
        assert!(!run.status.success()); // A reproduced panic remains a failing test.
        assert!(output(&run).contains("replay reproduced"), "{}", output(&run));
        let added: Vec<_> = manifests().into_iter().filter(|p| !before.contains(p)).collect();
        assert_eq!(added.len(), 1);
        let replay: Manifest = read(&added[0]).unwrap();
        assert_eq!(replay.trace.as_bytes(), recording.trace.as_bytes());
        assert_eq!(replay.outcome, recording.outcome);
        assert_eq!(std::fs::read(added[0].with_file_name("trace.ndjson")).unwrap(), recording.trace.as_bytes());
        replays.push(replay.trace);
    }
    assert_eq!(replays[0].as_bytes(), replays[1].as_bytes());
    let refuse = |manifest: &Manifest, component: &str| {
        let bad = dir.join("mismatch.json");
        let mut manifest = manifest.clone(); manifest.seal();
        std::fs::write(&bad, encoded(&manifest)).unwrap();
        let run = invoke(&bad);
        assert_eq!(run.status.code(), Some(2), "{}", output(&run));
        assert!(stderr(&run).contains(component), "{}", output(&run));
        assert!(!stderr(&run).contains('\u{1b}'), "unescaped manifest text: {}", output(&run));
        assert!(run.stdout.is_empty(), "candidate process started: {}", output(&run));
    };
    let mut old = recording.clone();
    old.format = "tide-replay/1".into();
    refuse(&old, &format!("re-record it with a fresh run of the same test with --seed {}", recording.seed));
    for component in ["format", "incomplete", "source", "compiler", "target", "profile", "options", "dependency", "dependency artifact", r"x\u{1b}\0", "configuration"] {
        let mut bad = recording.clone();
        match component {
            "format" => bad.format = "tide-replay/9".into(),
            "incomplete" => bad.complete = false,
            "source" => { bad.build.source.insert("file:demo.rs".into(), "changed".into()); }
            "compiler" => bad.build.compiler.push_str("changed"),
            "target" => bad.build.target.push_str("changed"),
            "profile" => bad.build.profile = "release".into(),
            "options" => bad.build.options.push("--changed".into()),
            "dependency" => { bad.build.dependencies[0].insert("std".into(), "changed".into()); }
            // The recorded names all match, but the build loads one more: the post-build check.
            "dependency artifact" => { bad.build.dependencies[1].pop_last().unwrap(); }
            // A recorded name the filesystem rejects (NUL) is reported escaped, never raw.
            r"x\u{1b}\0" => { bad.build.dependencies[1].insert("x\u{1b}\0".into(), "changed".into()); }
            "configuration" => { bad.config.insert("scheduler".into(), "changed".into()); }
            _ => unreachable!(),
        }
        refuse(&bad, component);
    }
    // A same-length dirty edit with its old mtime also invalidates the content snapshot.
    let source = dir.join("package/src/demo.rs");
    let modified = std::fs::metadata(&source).unwrap().modified().unwrap();
    std::fs::write(&source, original_source.replace("lost update", "lost updatf")).unwrap();
    std::fs::File::options().write(true).open(&source).unwrap()
        .set_times(std::fs::FileTimes::new().set_modified(modified)).unwrap();
    let refused = invoke(&recorded);
    assert_eq!(refused.status.code(), Some(2));
    assert!(stderr(&refused).contains("source mismatch"), "{}", output(&refused));
    assert!(refused.stdout.is_empty());
    let fresh = driver(&dir).args(["--crate-type=lib", "--test", "--tide-run-test", "simulation::loses_updates", "--seed", &recording.seed])
        .stdin(Stdio::null()).output().unwrap();
    assert!(!fresh.status.success());
    assert!(output(&fresh).contains("lost updatf"), "{}", output(&fresh));
    std::fs::write(source, original_source).unwrap();
    // Dependency identity covers the libraries the build loaded, as rustc's crate store has
    // them, under their names in the supplied directory, with physical roots normalized.
    let names = &recording.build.dependencies[1];
    assert!(!names.is_empty() && names.keys().all(|n| n.ends_with(".rlib") || n.ends_with(".so")), "{names:?}");
    assert!(names.keys().all(|n| std_dir().join(n).is_file()), "{names:?}");
    assert!(!names.contains_key("libtide_std.rlib"), "std has its own entry: {names:?}");
    let dependencies = dir.join("operator-dependencies");
    std::fs::create_dir(&dependencies).unwrap();
    for entry in std::fs::read_dir(std_dir()).unwrap() {
        let entry = entry.unwrap();
        if entry.file_type().unwrap().is_file() {
            std::fs::hard_link(entry.path(), dependencies.join(entry.file_name())).unwrap();
        }
    }
    let with_dependencies = |dependencies: &Path| {
        let original = driver(&dir);
        let mut command = Command::new(original.get_program());
        for arg in original.get_args() {
            command.arg(if arg == std_dir().as_os_str() { dependencies.as_os_str() } else { arg });
        }
        command.args(["--crate-type=lib", "--test", "--replay"]).arg(&recorded)
            .stdin(Stdio::null()).output().unwrap()
    };
    // An unrelated rebuild's churn: a new incremental cache, depfiles, and an unloaded library.
    std::fs::create_dir_all(dependencies.join("incremental/tide_std-1x2y/s-abc")).unwrap();
    std::fs::write(dependencies.join("incremental/tide_std-1x2y/s-abc/query-cache.bin"), b"churn").unwrap();
    std::fs::write(dependencies.join("tide_std.d"), b"rebuilt depfile").unwrap();
    std::fs::write(dependencies.join("libunrelated-0123456789abcdef.rlib"), b"not loaded").unwrap();
    // A library supplied as a symbolic link keeps its name in the directory, even beside an
    // earlier-sorting alias of the same file that rustc never loads.
    let (changed, removed) = (names.keys().next().unwrap(), names.keys().last().unwrap());
    std::fs::remove_file(dependencies.join(removed)).unwrap();
    std::os::unix::fs::symlink(std_dir().join(removed), dependencies.join(removed)).unwrap();
    std::os::unix::fs::symlink(std_dir().join(removed), dependencies.join(".alias")).unwrap();
    let churned = with_dependencies(&dependencies);
    assert!(output(&churned).contains("replay reproduced"), "{}", output(&churned));
    // A real change to a loaded library refuses before the build, one short line per library.
    let original_bytes = std::fs::read(dependencies.join(changed)).unwrap();
    std::fs::remove_file(dependencies.join(changed)).unwrap(); // Break the hard link first.
    std::fs::write(dependencies.join(changed), [&original_bytes[..], b"\0"].concat()).unwrap();
    if removed != changed { std::fs::remove_file(dependencies.join(removed)).unwrap(); }
    let refused = with_dependencies(&dependencies);
    assert_eq!(refused.status.code(), Some(2), "{}", output(&refused));
    assert!(refused.stdout.is_empty());
    let message = stderr(&refused);
    assert!(message.contains("replay dependency artifact mismatch"), "{message}");
    let digest = |name: &str| names[name][..12].to_string();
    let line = message.lines().find(|l| l.trim_start().starts_with(&format!("{changed}: "))).expect("changed line");
    assert!(line.contains(&format!("recorded {}, current ", digest(changed))), "{line}");
    assert!(!line.contains(&names[changed.as_str()]) && line.len() < changed.len() + 60, "{line}");
    if removed != changed {
        assert!(message.contains(&format!("{removed}: recorded {}, current missing", digest(removed))), "{message}");
    }
    assert!(message.lines().count() <= 3 && !message.contains('{'), "{message}");
    let combined = driver(&dir).args(["--crate-type=lib", "--test", "--seed", "0", "--replay"]).arg(&recorded)
        .stdin(Stdio::null()).output().unwrap();
    assert_eq!(combined.status.code(), Some(2));
    assert!(stderr(&combined).contains("configuration override"));

    // Parameter tapes belong to the original evaluation, not the later shrink result.
    let parameters = package("replay-parameters", r#"
#[sim_test(seeds = 1)]
fn parameters(sim: std::sim::SimRoot, values: Vec<u16>) {
    sim.world.args(["demo", "quoted argument"]);
    sim.world.env_var("MODE", "test");
    sim.world.stdin([0, 255, 10]);
    sim.world.file("/config", [0, 255]);
    println!("{:?}", values);
    sim.tasks.checkpoint().unwrap();
    panic!("parameter witness");
}
#[test]
#[should_panic]
fn expected_panic(value: bool) {
    let sim = Root::sim(47);
    println!("value={value}");
    sim.tasks.checkpoint().unwrap();
    panic!("expected witness");
}
#[test]
fn sequential_roots() {
    { let sim = Root::sim(17); sim.tasks.checkpoint().unwrap(); }
    { let sim = Root::sim(29); sim.tasks.checkpoint().unwrap(); }
    panic!("plain witness");
}
"#);
    for name in ["parameters", "sequential_roots", "expected_panic"] {
        let fresh = driver(&parameters).args(["--crate-type=lib", "--test", "--tide-run-test", name, "--seed", "47"])
            .stdin(Stdio::null()).output().unwrap();
        assert_eq!(fresh.status.success(), name == "expected_panic", "{}", output(&fresh));
        let path = std::fs::read_dir(parameters.join("out/target/sim")).unwrap()
            .filter_map(|e| { let p = e.unwrap().path().join("run.manifest.json"); p.is_file().then_some(p) })
            .find(|p| read::<Manifest>(p).unwrap().test == name).unwrap();
        let manifest: Manifest = read(&path).unwrap();
        if name == "sequential_roots" {
            assert_eq!(manifest.roots.iter().map(|r| r.seed.as_str()).collect::<Vec<_>>(), ["17", "29"]);
        } else if name == "parameters" {
            assert!(!manifest.choices.is_empty());
            assert_eq!(manifest.roots[0].inputs.len(), 5);
            assert_eq!(manifest.roots[0].inputs[4].value, [0, 255]);
        }
        let run = driver(&parameters).args(["--crate-type=lib", "--test", "--replay"]).arg(&path)
            .stdin(Stdio::null()).output().unwrap();
        assert_eq!(run.status.success(), name == "expected_panic", "{}", output(&run));
        assert!(output(&run).contains("replay reproduced"), "{}", output(&run));
        if name == "parameters" || name == "expected_panic" {
            for extra in [false, true] {
                let mut bad = manifest.clone();
                if extra { bad.choices.push(replay_wire::Choice { kind: "bool".into(), bound: "1".into(), value: "0".into() }); }
                else { bad.choices.clear(); }
                bad.seal(); std::fs::write(parameters.join("bad.json"), encoded(&bad)).unwrap();
                let run = driver(&parameters).args(["--crate-type=lib", "--test", "--replay"]).arg(parameters.join("bad.json"))
                    .stdin(Stdio::null()).output().unwrap();
                assert_eq!(run.status.code(), Some(2), "{}", output(&run));
                assert!(output(&run).contains(if extra { "tape has leftovers" } else { "tape exhausted" }), "{}", output(&run));
                assert!(!output(&run).contains("replay reproduced"));
            }
            for component in ["trace", "outcome"] {
                let mut bad = manifest.clone();
                if component == "trace" { bad.trace.push_str("extra event\n"); }
                else { bad.outcome = "different outcome".into(); }
                bad.seal(); std::fs::write(parameters.join("bad.json"), encoded(&bad)).unwrap();
                let run = driver(&parameters).args(["--crate-type=lib", "--test", "--replay"]).arg(parameters.join("bad.json"))
                    .stdin(Stdio::null()).output().unwrap();
                assert_eq!(run.status.code(), Some(2), "{}", output(&run));
                assert!(output(&run).contains(if component == "trace" { "first differing trace line" } else { "outcome mismatch" }), "{}", output(&run));
                assert!(!output(&run).contains("replay reproduced"));
            }
        }
    }

    let paired = package("replay-paired", r#"
#[sim_test(seeds = 1)]
fn paired(sim: std::sim::SimRoot) {
    println!("{}", tide_candidate::value());
    sim.tasks.checkpoint().unwrap();
    panic!("paired witness");
}
"#);
    std::fs::create_dir_all(paired.join("candidate/src")).unwrap();
    let library = paired.join("candidate/src/lib.rs");
    std::fs::write(&library, "pub fn value() -> u32 { 1 }").unwrap();
    let fresh = paired_test_driver(&paired).args(["--tide-run-test", "paired"]).output().unwrap();
    assert!(!fresh.status.success());
    let path = std::fs::read_dir(paired.join("out/target/sim")).unwrap()
        .map(|e| e.unwrap().path().join("run.manifest.json")).find(|p| p.is_file()).unwrap();
    let replay = paired_test_driver(&paired).arg("--replay").arg(&path).output().unwrap();
    assert!(output(&replay).contains("replay reproduced"), "{}", output(&replay));
    std::fs::write(&library, "pub fn value() -> u32 { 2 }").unwrap();
    let refused = paired_test_driver(&paired).arg("--replay").arg(&path).output().unwrap();
    assert_eq!(refused.status.code(), Some(2));
    assert!(stderr(&refused).contains("paired-library source mismatch"), "{}", output(&refused));
    assert!(refused.stdout.is_empty());

    // The relocated-dependency replay above left a second entry with the same receipt: the
    // identity no longer depends on dependency roots, so corrupt every matching entry.
    let executables: Vec<_> = std::fs::read_dir(dir.join("out/.tide-cache")).unwrap()
        .map(|e| e.unwrap().path()).filter(|entry| {
            read::<replay_wire::Receipt>(&entry.join("build.json")).is_ok_and(|r| r.build == recording.build)
        }).map(|entry| entry.join("executable")).collect();
    assert!(!executables.is_empty());
    let originals: Vec<_> = executables.iter().map(|e| std::fs::read(e).unwrap()).collect();
    for (executable, original) in executables.iter().zip(&originals) {
        std::fs::write(executable, [&original[..], &[0]].concat()).unwrap();
    }
    let refused = invoke(&recorded);
    assert_eq!(refused.status.code(), Some(2));
    assert!(stderr(&refused).contains("executable/receipt mismatch"), "{}", output(&refused));
    assert!(refused.stdout.is_empty());
    for (executable, original) in executables.iter().zip(&originals) {
        std::fs::write(executable, original).unwrap();
    }

    let relocated = package("replay-relocated", original_source);
    let replay = driver(&relocated).args(["--crate-type=lib", "--test", "--replay"]).arg(&recorded)
        .stdin(Stdio::null()).output().unwrap();
    assert!(output(&replay).contains("replay reproduced"), "{}", output(&replay));
}

/// Compiles `source` with rustc outside the driver: a program that never passed admission.
fn unadmitted(dir: &Path, source: &str) -> PathBuf {
    let (code, binary) = (dir.join("unadmitted.rs"), dir.join("unadmitted"));
    std::fs::write(&code, source).unwrap();
    plain_rustc(&code, &binary);
    binary
}

/// Replaces the cached admitted test executable with `binary`, and the receipt that names its
/// digest, as code that reaches RUN-4's holes 1 or 5 could.
fn swap_cached_executable(dir: &Path, binary: &Path) {
    use replay_wire::{Receipt, encoded, read};
    let entry = std::fs::read_dir(dir.join("out/.tide-cache")).unwrap()
        .map(|e| e.unwrap().path()).find(|p| p.join("executable").is_file()).unwrap();
    let bytes = std::fs::read(binary).unwrap();
    std::fs::write(entry.join("executable"), &bytes).unwrap();
    let mut receipt: Receipt = read(&entry.join("build.json")).unwrap();
    receipt.executable = replay_wire::digest(&bytes);
    std::fs::write(entry.join("build.json"), encoded(&receipt)).unwrap();
}

/// Runs `command` (stderr piped, stdout as configured) and fails if it outlives `seconds`.
fn bounded(command: &mut Command, seconds: u64) -> (std::process::ExitStatus, String) {
    use std::io::Read;
    let mut child = command.stdin(Stdio::null()).stderr(Stdio::piped()).spawn().unwrap();
    let mut pipe = child.stderr.take().unwrap();
    let reader = std::thread::spawn(move || { let mut s = String::new(); let _ = pipe.read_to_string(&mut s); s });
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(seconds);
    let status = loop {
        if let Some(status) = child.try_wait().unwrap() { break status; }
        if std::time::Instant::now() > deadline {
            let _ = child.kill();
            let _ = child.wait();
            panic!("hang guard: the driver outlived {seconds} s");
        }
        std::thread::sleep(std::time::Duration::from_millis(20));
    };
    (status, reader.join().unwrap())
}

/// RUN-3 kernel denial (fx-ip4x): the kernel, not admission, stops a test child that reaches
/// outside its grants. Since `TD0206` rejects the fx-6xi2 forging pattern at admission, the child
/// here never passed admission at all: a plain Rust program replaces the cached admitted
/// executable. It truncates its standard output, reads and writes paths outside the run, reads
/// through a planted `target/sim` link a canary and another run's artifact, connects to a
/// listener, and writes a directory named like another run's published artifact. Run directly
/// it reaches all of them; started by `--tide-run-test` it gets Landlock's `EACCES` for the
/// paths, seccomp's `EPERM` for the socket, and a pipe the harness owns as standard output
/// (`EINVAL`); the planted link is neither followed nor deleted, and the published artifact keeps
/// its bytes. Needs a host whose level is `full`.
#[test]
fn run3_kernel_denies_an_unadmitted_test_child_paths_and_sockets() {
    let dir = package("run3-kernel-denial", "#[test]\nfn reaches() {}\n");
    let run = || {
        let mut command = driver(&dir);
        command.args(["--crate-type=lib", "--test", "--tide-run-test", "reaches"]);
        command
    };
    let (status, admitted) = bounded(&mut run().stdout(Stdio::null()), 300);
    assert!(status.success(), "{admitted}");
    let level = admitted.lines().find(|l| l.starts_with("sandbox: ")).expect("the run reports its level").to_owned();
    assert!(level.starts_with("sandbox: full (Linux ") && level.contains("Landlock ABI"), "{level}");

    let canary = dir.join("canary.txt");
    std::fs::write(&canary, "canary").unwrap();
    let escaped = dir.join("escaped.txt");
    // `target/sim` is planted as a link to a directory holding a canary and another run's artifact.
    let elsewhere = dir.join("elsewhere");
    std::fs::create_dir_all(elsewhere.join("other-run")).unwrap();
    std::fs::write(elsewhere.join("canary.txt"), "canary").unwrap();
    std::fs::write(elsewhere.join("other-run/run.manifest.json"), "{}").unwrap();
    let sim = dir.join("out/target/sim");
    std::fs::remove_dir_all(&sim).unwrap();
    std::os::unix::fs::symlink(&elsewhere, &sim).unwrap();
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let reach = unadmitted(&dir, &format!(r#"
fn main() {{
    // SAFETY: standard output stays open; the handle is never dropped.
    let out = std::mem::ManuallyDrop::new(unsafe {{ <std::fs::File as std::os::fd::FromRawFd>::from_raw_fd(1) }});
    let truncate = out.set_len(0).map_err(|e| e.raw_os_error());
    let read = std::fs::read({canary:?}).map(|b| b.len()).map_err(|e| e.raw_os_error());
    let write = std::fs::write({escaped:?}, "out").map_err(|e| e.raw_os_error());
    let socket = std::net::TcpStream::connect({address:?}).map(drop).map_err(|e| e.raw_os_error());
    let sim = std::fs::read({sim_canary:?}).map(|b| b.len()).map_err(|e| e.raw_os_error());
    let other = std::fs::read({other:?}).map(|b| b.len()).map_err(|e| e.raw_os_error());
    println!("RUN3 truncate={{truncate:?}} read={{read:?}} write={{write:?}} socket={{socket:?}} sim={{sim:?}} other={{other:?}}");
    // In the harness, claim another run's published name inside the writable directory.
    if let Ok(context) = std::env::var("__TIDE_REPLAY_CONTEXT") {{
        let context = std::fs::read_to_string(context).unwrap();
        let directory = context.split("\"directory\":\"").nth(1).unwrap().split('"').next().unwrap();
        let forged = std::path::Path::new(directory).join("other-run");
        std::fs::create_dir_all(&forged).unwrap();
        std::fs::write(forged.join("run.manifest.json"), "forged").unwrap();
        println!("FORGED");
    }}
}}
"#, address = listener.local_addr().unwrap().to_string(), sim_canary = sim.join("canary.txt"),
        other = sim.join("other-run/run.manifest.json")));
    // Standard output goes to a file that already holds a line, opened for appending.
    let redirected = |name: &str| {
        let path = dir.join(name);
        std::fs::write(&path, "kept\n").unwrap();
        (path.clone(), std::fs::OpenOptions::new().append(true).open(path).unwrap())
    };
    let report = |path: &Path| std::fs::read_to_string(path).unwrap().lines()
        .find(|l| l.starts_with("RUN3 ")).map(str::to_owned).unwrap_or_default();
    // Control: outside the sandbox, the same program reaches every canary.
    let (direct_out, file) = redirected("direct.out");
    checked(Command::new(&reach).stdout(file));
    assert_eq!(report(&direct_out),
        "RUN3 truncate=Ok(()) read=Ok(6) write=Ok(()) socket=Ok(()) sim=Ok(6) other=Ok(2)");
    assert!(!std::fs::read_to_string(&direct_out).unwrap().starts_with("kept"));
    drop(listener.accept().unwrap());
    std::fs::remove_file(&escaped).unwrap();

    swap_cached_executable(&dir, &reach);
    listener.set_nonblocking(true).unwrap();
    let (sandboxed_out, file) = redirected("sandboxed.out");
    let (_, errors) = bounded(run().stdout(file), 300);
    assert!(errors.contains(&level), "{errors}");
    assert_eq!(report(&sandboxed_out),
        "RUN3 truncate=Err(Some(22)) read=Err(Some(13)) write=Err(Some(13)) socket=Err(Some(1)) sim=Err(Some(13)) other=Err(Some(13))",
        "{errors}");
    assert!(std::fs::read_to_string(&sandboxed_out).unwrap().starts_with("kept\n"));
    assert!(!escaped.exists());
    assert_eq!(listener.accept().unwrap_err().kind(), std::io::ErrorKind::WouldBlock);
    // The planted link is neither followed nor deleted: publication fails and says why.
    assert!(std::fs::symlink_metadata(&sim).unwrap().file_type().is_symlink());
    assert!(errors.contains("is a symbolic link or not a directory"), "{errors}");
    let mut planted: Vec<_> = std::fs::read_dir(&elsewhere).unwrap().map(|e| e.unwrap().file_name()).collect();
    planted.sort();
    assert_eq!(planted, ["canary.txt", "other-run"]);

    // A real published artifact keeps its bytes when the child claims its name.
    std::fs::remove_file(&sim).unwrap();
    std::fs::create_dir_all(sim.join("other-run")).unwrap();
    std::fs::write(sim.join("other-run/run.manifest.json"), "original").unwrap();
    let (claimed_out, file) = redirected("claimed.out");
    let (status, errors) = bounded(run().stdout(file), 300);
    assert!(status.success(), "{errors}");
    assert!(std::fs::read_to_string(&claimed_out).unwrap().contains("FORGED"), "{errors}");
    assert_eq!(std::fs::read_to_string(sim.join("other-run/run.manifest.json")).unwrap(), "original");
}

/// The ELF interpreter (`PT_INTERP`) that `binary` names.
fn interpreter(binary: &Path) -> String {
    let elf = std::fs::read(binary).unwrap();
    let u64_at = |at: usize| u64::from_le_bytes(elf[at..at + 8].try_into().unwrap()) as usize;
    let u16_at = |at: usize| u16::from_le_bytes(elf[at..at + 2].try_into().unwrap()) as usize;
    let (table, size, count) = (u64_at(0x20), u16_at(0x36), u16_at(0x38));
    let header = (0..count).map(|i| table + i * size)
        .find(|&h| u32::from_le_bytes(elf[h..h + 4].try_into().unwrap()) == 3)
        .expect("a dynamically linked executable");
    let (offset, len) = (u64_at(header + 8), u64_at(header + 32));
    String::from_utf8(elf[offset..offset + len - 1].to_vec()).unwrap()
}

/// RUN-3 red team (fx-qlek.5): what the kernel denies a test child that reached RUN-4's holes 1
/// or 5, as in the test above. A `.init_array` constructor, which runs before `main`, cannot read
/// a canary or fork. `main` cannot execute `/bin/sh` (Landlock, `EACCES`; `execveat`, seccomp's
/// `EPERM`), fork, vfork or clone a process, set up io_uring, trace, make a memfd or a namespace,
/// signal a process of its user outside the sandbox (`EPERM`), open
/// `/proc/self/mem` (`EACCES`), make a socket pair, raise a limit, or read its own memory through
/// `process_vm_readv` (`EPERM`); `clone3` reports `ENOSYS`. Last, it executes its own ELF
/// interpreter, which Landlock lets it run, on `/bin/sh`: the loader starts and fails to open the
/// shell. Run directly, the same binary reads its memory through `process_vm_readv` and the loader
/// runs the shell. Needs a host whose level is `full`.
#[test]
fn run3_kernel_denies_exec_fork_io_uring_ptrace_and_constructors() {
    let dir = package("run3-red-team", "#[test]\nfn reaches() {}\n");
    let run = || {
        let mut command = driver(&dir);
        command.args(["--crate-type=lib", "--test", "--tide-run-test", "reaches"]);
        command
    };
    let (status, admitted) = bounded(&mut run().stdout(Stdio::null()), 300);
    assert!(status.success(), "{admitted}");
    assert!(admitted.lines().any(|l| l.starts_with("sandbox: full (Linux ")), "{admitted}");
    let canary = dir.join("canary.txt");
    std::fs::write(&canary, "canary").unwrap();
    // The host's interpreter, which the child names too and the sandbox grants it.
    let loader = interpreter(&std::env::current_exe().unwrap());
    let reach = unadmitted(&dir, &format!(r#"
use std::sync::atomic::{{AtomicI32, Ordering}};
unsafe extern "C" {{
    fn syscall(n: i64, ...) -> i64;
    fn execve(path: *const u8, argv: *const *const u8, envp: *const *const u8) -> i32;
}}
fn result(value: i64) -> Result<i64, Option<i32>> {{
    if value < 0 {{ Err(std::io::Error::last_os_error().raw_os_error()) }} else {{ Ok(value) }}
}}
/// A process the call made exits at once.
fn call(n: i64, a: i64, b: i64, c: i64, d: i64, e: i64) -> Result<i64, Option<i32>> {{
    // SAFETY: each call passes only numbers and pointers to live buffers.
    let value = unsafe {{ syscall(n, a, b, c, d, e, 0i64) }};
    if value == 0 && matches!(n, 56..=58) {{ unsafe {{ syscall(231, 0) }}; }}
    result(value)
}}
fn errno(r: Result<i64, Option<i32>>) -> i32 {{ r.err().flatten().unwrap_or(0) }}
static CTOR_READ: AtomicI32 = AtomicI32::new(-1);
static CTOR_FORK: AtomicI32 = AtomicI32::new(-1);
extern "C" fn early() {{
    let read = std::fs::read({canary:?}).map(drop).map_err(|e| e.raw_os_error().unwrap_or(0));
    CTOR_READ.store(read.err().unwrap_or(0), Ordering::SeqCst);
    CTOR_FORK.store(errno(call(57, 0, 0, 0, 0, 0)), Ordering::SeqCst);
}}
#[used]
#[unsafe(link_section = ".init_array")]
static CONSTRUCTOR: extern "C" fn() = early;
fn main() {{
    let sandboxed = std::env::var_os("__TIDE_REPLAY_CONTEXT").is_some();
    let argv = |args: &[&std::ffi::CStr]| -> Vec<*const u8> {{
        args.iter().map(|a| a.as_ptr().cast()).chain([std::ptr::null()]).collect()
    }};
    // `process_vm_readv` of one byte of this process's own memory, which needs no permission.
    let source = std::hint::black_box(7u8);
    let mut copy = 0u8;
    let local = [&mut copy as *mut u8 as i64, 1];
    let remote = [&source as *const u8 as i64, 1];
    let vmread = call(310, i64::from(std::process::id()), local.as_ptr() as i64, 1, remote.as_ptr() as i64, 1);
    let copy = std::hint::black_box(copy);
    if !sandboxed {{
        println!("CONTROL vmread={{vmread:?}} copy={{copy}}");
    }} else {{
        let shell = argv(&[c"/bin/sh", c"-c", c"echo ESCAPED"]);
        // SAFETY: NUL-terminated strings and arrays.
        let exec = result(unsafe {{ execve(shell[0], shell.as_ptr(), std::ptr::null()) }}.into());
        let mut params = [0u8; 120];
        let mut fds = [0i32; 2];
        let limit = [1u64 << 20, 1u64 << 20];
        // A process outside the sandbox, of the same user: this test.
        let outside: i64 = {outside};
        let mem = std::fs::OpenOptions::new().read(true).write(true).open("/proc/self/mem")
            .map(drop).map_err(|e| e.raw_os_error());
        println!("RUN3B exec={{exec:?}} execveat={{:?}} fork={{:?}} vfork={{:?}} clone={{:?}} clone3={{:?}} io_uring={{:?}} ptrace={{:?}} memfd={{:?}} unshare={{:?}} kill={{:?}} tgkill={{:?}} mem={{mem:?}} socketpair={{:?}} prlimit={{:?}} vmread={{vmread:?}} ctor_read={{}} ctor_fork={{}}",
            call(322, -100, c"/bin/sh".as_ptr() as i64, shell.as_ptr() as i64, 0, 0),
            call(57, 0, 0, 0, 0, 0),
            call(58, 0, 0, 0, 0, 0),
            call(56, 17, 0, 0, 0, 0),
            call(435, 0, 0, 0, 0, 0),
            call(425, 8, params.as_mut_ptr() as i64, 0, 0, 0),
            call(101, 0, 0, 0, 0, 0),
            call(319, c"x".as_ptr() as i64, 0, 0, 0, 0),
            call(272, 0x1000_0000, 0, 0, 0, 0),
            call(62, outside, 0, 0, 0, 0),
            call(234, outside, outside, 0, 0, 0),
            call(53, 1, 1, 0, fds.as_mut_ptr() as i64, 0),
            call(302, 0, 7, limit.as_ptr() as i64, 0, 0),
            CTOR_READ.load(Ordering::SeqCst), CTOR_FORK.load(Ordering::SeqCst));
    }}
    use std::io::Write;
    std::io::stdout().flush().unwrap();
    let loader = argv(&[c{loader:?}, c"/bin/sh", c"-c", c"echo ESCAPED"]);
    // SAFETY: as above.
    unsafe {{ execve(loader[0], loader.as_ptr(), std::ptr::null()) }};
    println!("LOADER {{:?}}", std::io::Error::last_os_error().raw_os_error());
}}
"#, outside = std::process::id()));
    assert_eq!(interpreter(&reach), loader);
    // Control: run directly, the self-read succeeds and the loader runs the shell.
    let control = dir.join("control.out");
    checked(Command::new(&reach).stdout(std::fs::File::create(&control).unwrap()));
    let control = std::fs::read_to_string(&control).unwrap();
    assert!(control.contains("CONTROL vmread=Ok(1) copy=7\n") && control.contains("ESCAPED\n")
        && !control.contains("LOADER"), "{control}");

    swap_cached_executable(&dir, &reach);
    let path = dir.join("red-team.out");
    let file = std::fs::File::create(&path).unwrap();
    let (status, errors) = bounded(run().stdout(file), 300);
    let out = std::fs::read_to_string(&path).unwrap();
    let report = out.lines().find(|l| l.starts_with("RUN3B ")).unwrap_or_default();
    assert_eq!(report,
        "RUN3B exec=Err(Some(13)) execveat=Err(Some(1)) fork=Err(Some(1)) vfork=Err(Some(1)) clone=Err(Some(1)) clone3=Err(Some(38)) \
         io_uring=Err(Some(1)) ptrace=Err(Some(1)) memfd=Err(Some(1)) unshare=Err(Some(1)) \
         kill=Err(Some(1)) tgkill=Err(Some(1)) mem=Err(Some(13)) socketpair=Err(Some(1)) \
         prlimit=Err(Some(1)) vmread=Err(Some(1)) ctor_read=13 ctor_fork=1",
        "{out}\n{errors}");
    // The loader started (no `LOADER` return) and failed on the shell it may not read.
    assert!(!status.success(), "{out}\n{errors}");
    assert!(!out.contains("LOADER") && !out.contains("ESCAPED") && !errors.contains("ESCAPED"),
        "{out}\n{errors}");
    assert!(errors.contains("/bin/sh: error while loading shared libraries: /bin/sh: cannot open \
        shared object file: Permission denied"), "{out}\n{errors}");
}

/// The harness owns a driver-run simulation's per-evaluation limit and its crash report, since
/// the evaluation runs in the sandboxed child itself (RUN-3, RUN-4 hole 6).
#[test]
fn run3_harness_times_out_and_reports_signalled_simulations() {
    let dir = package("run3-harness-limits", r#"
#[sim_test(seeds = 1)]
fn spins(sim: std::sim::SimRoot) { drop(sim); loop {} }
fn deep(n: u64) -> u64 { if n == u64::MAX { n } else { deep(n + 1) + 1 } }
#[sim_test(seeds = 1)]
fn overflows(sim: std::sim::SimRoot) { drop(sim); assert!(deep(0) > 0); }
"#);
    let run = |name: &str, extra: &[&str]| bounded(driver(&dir)
        .args(["--crate-type=lib", "--test", "--tide-run-test", name, "--seed", "47"]).args(extra)
        .stdout(Stdio::null()), 300);
    let (status, errors) = run("spins", &["--tide-eval-timeout", "2"]);
    assert!(!status.success());
    assert!(errors.contains("test spins failed: per-evaluation wall-clock timeout (last started seed: 47)"), "{errors}");
    let (status, errors) = run("overflows", &[]);
    assert!(!status.success());
    assert!(errors.contains("test overflows failed: child terminated by signal")
        && errors.contains("(last started seed: 47)"), "{errors}");
}

/// A consumer that stops reading the driver's standard output delays neither seed observation
/// nor the limit's report: the child floods its stdout and spins, and the driver still stops it,
/// reports on standard error, and exits.
#[test]
fn run3_stalled_stdout_does_not_hold_the_limit() {
    let dir = package("run3-stalled-stdout", "#[sim_test(seeds = 1)]\nfn floods(sim: std::sim::SimRoot) { drop(sim); }\n");
    let run = || {
        let mut command = driver(&dir);
        command.args(["--crate-type=lib", "--test", "--tide-run-test", "floods", "--tide-eval-timeout", "2"]);
        command
    };
    let (status, errors) = bounded(run().stdout(Stdio::null()), 300);
    assert!(status.success(), "{errors}");
    let flood = unadmitted(&dir, r#"
fn main() {
    println!("\nTIDE_SIM_SEED_START 47");
    for i in 0..100_000 { println!("flood line {i} ........................................................"); }
    loop { std::thread::sleep(std::time::Duration::from_millis(100)); }
}
"#);
    swap_cached_executable(&dir, &flood);
    let (stalled, unread) = std::io::pipe().unwrap();
    let (status, errors) = bounded(run().stdout(unread), 120);
    drop(stalled);
    assert!(!status.success());
    assert!(errors.contains("test floods failed: per-evaluation wall-clock timeout (last started seed: 47)"), "{errors}");
}

/// The same for standard error: once the run's `sandbox:` line is read, nobody reads the driver's
/// stderr, the child floods it and spins, and the driver still stops it and exits with failure.
#[test]
fn run3_stalled_stderr_does_not_hold_the_limit() {
    use std::io::BufRead;
    let dir = package("run3-stalled-stderr", "#[sim_test(seeds = 1)]\nfn floods(sim: std::sim::SimRoot) { drop(sim); }\n");
    let run = || {
        let mut command = driver(&dir);
        command.args(["--crate-type=lib", "--test", "--tide-run-test", "floods", "--tide-eval-timeout", "2"]);
        command
    };
    let (status, errors) = bounded(run().stdout(Stdio::null()), 300);
    assert!(status.success(), "{errors}");
    let flood = unadmitted(&dir, r#"
fn main() {
    println!("\nTIDE_SIM_SEED_START 47");
    for i in 0..100_000 { eprintln!("flood line {i} ........................................................"); }
    loop { std::thread::sleep(std::time::Duration::from_millis(100)); }
}
"#);
    swap_cached_executable(&dir, &flood);
    let mut child = run().stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn().unwrap();
    let mut stdout = child.stdout.take().unwrap();
    let drained = std::thread::spawn(move || std::io::copy(&mut stdout, &mut std::io::sink()));
    // Read up to the level line, then hold the pipe open without reading it.
    let mut stderr = std::io::BufReader::new(child.stderr.take().unwrap());
    let mut line = String::new();
    while stderr.read_line(&mut line).unwrap() != 0 && !line.starts_with("sandbox: ") {
        line.clear();
    }
    assert!(line.starts_with("sandbox: "), "{line}");
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(120);
    let status = loop {
        if let Some(status) = child.try_wait().unwrap() { break status; }
        if std::time::Instant::now() > deadline {
            let _ = child.kill();
            let _ = child.wait();
            panic!("hang guard: the driver outlived 120 s with its stderr stalled");
        }
        std::thread::sleep(std::time::Duration::from_millis(20));
    };
    drop(stderr);
    drained.join().unwrap().unwrap();
    assert!(!status.success());
}

/// RUN-3's new session takes the child out of the caller's process group, so the child instead
/// dies with the driver (`PR_SET_PDEATHSIG`): killing the driver's process group, as `timeout`
/// and a terminal interrupt do, leaves no sandboxed child running.
#[test]
fn run3_child_dies_with_the_driver() {
    use std::io::BufRead;
    use std::os::unix::process::CommandExt;
    use std::time::{Duration, Instant};
    let dir = package("run3-child-lifetime", "#[test]\nfn spins() {}\n");
    let run = || {
        let mut command = driver(&dir);
        command.args(["--crate-type=lib", "--test", "--tide-run-test", "spins"]);
        command
    };
    let (status, errors) = bounded(run().stdout(Stdio::null()), 300);
    assert!(status.success(), "{errors}");
    let spin = unadmitted(&dir, r#"
fn main() {
    println!("PID {}", std::process::id());
    loop { std::hint::spin_loop(); }
}
"#);
    swap_cached_executable(&dir, &spin);
    let mut driven = run().process_group(0).stdin(Stdio::null()).stdout(Stdio::piped())
        .stderr(Stdio::null()).spawn().unwrap();
    let group = driven.id() as i32;
    let stdout = driven.stdout.take().unwrap();
    let (send, receive) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        for line in std::io::BufReader::new(stdout).lines() {
            if let Some(pid) = line.ok().and_then(|l| l.strip_prefix("PID ")?.trim().parse::<i32>().ok()) {
                let _ = send.send(pid);
            }
        }
    });
    // SAFETY: signals only the driver's own process group.
    let kill_group = || unsafe { libc::kill(-group, libc::SIGKILL) };
    let Ok(child) = receive.recv_timeout(Duration::from_secs(120)) else {
        kill_group();
        panic!("hang guard: the sandboxed child never reported its pid");
    };
    kill_group();
    let _ = driven.wait();
    // Gone, or a zombie awaiting its new parent's reap.
    let gone = || std::fs::read_to_string(format!("/proc/{child}/stat"))
        .map_or(true, |stat| stat.rsplit(')').next().is_some_and(|rest| rest.trim_start().starts_with('Z')));
    let deadline = Instant::now() + Duration::from_secs(10);
    while !gone() {
        if Instant::now() > deadline {
            // SAFETY: the hang guard ends the child it found, not only the driver.
            unsafe { libc::kill(child, libc::SIGKILL) };
            panic!("the sandboxed child {child} outlived its driver");
        }
        std::thread::sleep(Duration::from_millis(20));
    }
}

