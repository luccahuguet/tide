#!/usr/bin/env python3
"""Small contract checks for rendering, structure, budgets, and child timeouts."""
import json
from pathlib import Path
import re
import subprocess
import sys
import tempfile
import time
import unittest
from unittest.mock import patch

import harness as h


class HarnessTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        (h.ROOT / "target").mkdir(exist_ok=True)

    def test_render_every_arm_without_other_tasks_or_labels(self):
        for task in h.TASKS:
            for arm in ["rust", "tide"]:
                text = h.task_text(task, arm)
                self.assertEqual(text.count("### "), 1)
                self.assertIn(f"### {task}.", text)
                self.assertNotIn("- Tide:", text)
                self.assertNotIn("- Rust:", text)
                self.assertIn("Checked cases", text)
                self.assertIn("- Write it in " + ("Tide." if arm == "tide" else "Rust 2024."), text)
        rust = h.task_text("B9", "rust")
        tide = h.task_text("B9", "tide")
        self.assertNotIn("write_synced(", rust)
        self.assertNotIn("sync_all(", tide)
        self.assertIn("sync_dir(`", tide)

    def test_structure_positive_and_negative_boundaries(self):
        for task in h.TASKS:
            for arm in ["rust", "tide"]:
                text = h.source(task, arm).read_text()
                self.assertTrue(h.structure(task, arm, text), (task, arm))
                self.assertFalse(h.integrity_flags(text), (task, arm))
        self.assertFalse(h.structure("C7", "tide", "#[requires(x)] #[ensures(y)]"))
        self.assertFalse(h.structure("C2", "tide", "std::par par_sort"))
        self.assertTrue(h.structure("C3", "rust", "// sync_channel("))
        self.assertFalse(h.structure("B7", "tide", "#[pure]\nfn other() {}\npub fn plan() {}"))

    def test_preparation_snapshots_and_refuses_run_21(self):
        with tempfile.TemporaryDirectory(dir=h.ROOT / "target") as tmp:
            root = Path(tmp)
            with patch.object(h, "STATE", root / "state"):
                session = h.prepare("A2", "rust", root / "writer", {"sandbox": False, "sysroot": "/toolchain", "vendor": "/vendor"})
                self.assertFalse((root / "writer/references").exists())
                self.assertFalse((root / "writer/check").exists())
                self.assertFalse((root / "writer/test").exists())
                (root / "writer/src/lib.rs").write_bytes(b"first\r\n")
                green = [{"code": 0, "stdout": "", "stderr": ""}]
                with patch.object(h, "checks", return_value=green):
                    self.assertEqual(h.command(session, "check"), 0)
                    (root / "writer/src/lib.rs").write_text("final")
                    self.assertEqual(h.command(session, "check"), 0)
                self.assertEqual((session / "snapshot-01/source.rs").read_bytes(), b"first\r\n")
                self.assertEqual((session / "snapshot-02/source.rs").read_text(), "final")
                reference = h.prepare("A2", "rust", root / "reference-root/A2",
                                      {"sandbox": False, "sysroot": "/toolchain", "vendor": "/vendor"}, reference=True)
                self.assertTrue(reference.is_relative_to(root / "reference-root"))
                self.assertFalse(reference.is_relative_to(h.STATE / "sessions"))
                state = json.loads((session / "session.json").read_text())
                state["runs"] = 20
                (session / "session.json").write_text(json.dumps(state))
                self.assertEqual(h.command(session, "check"), 2)
                state["runs"] = 0
                state["started"] = time.time() - 2401
                (session / "session.json").write_text(json.dumps(state))
                self.assertEqual(h.command(session, "check"), 2)

    def test_timeout_and_clean_environment(self):
        with tempfile.TemporaryDirectory(dir=h.ROOT / "target") as tmp:
            root = Path(tmp)
            config = {"sandbox": False, "sysroot": "/toolchain"}
            result = h.run([sys.executable, "-c", "import time; time.sleep(10)"], config, root, timeout=0.05)
            self.assertEqual(result["code"], 124)
            result = h.run([sys.executable, "-c", "import os,sys; print(os.environ.get('E1_SECRET')); print(sys.stdin.read())"], config, root)
            self.assertEqual(result["code"], 0)
            self.assertEqual(result["stdout"], "None\n\n")
            result = h.run(["/usr/bin/env"], config, root, env={})
            self.assertEqual(result["stdout"], "")

    def test_invalid_source_attempts_are_recorded_and_recover(self):
        for broken in ["missing", "directory", "escaping", "loop", "non-utf8"]:
            with self.subTest(broken=broken), tempfile.TemporaryDirectory(dir=h.ROOT / "target") as tmp:
                root = Path(tmp)
                with patch.object(h, "STATE", root / "state"):
                    session = h.prepare("A2", "rust", root / "writer",
                                        {"sandbox": False, "sysroot": "/toolchain", "vendor": "/vendor"})
                    entry = root / "writer/src/lib.rs"
                    entry.unlink()
                    if broken == "directory": entry.mkdir()
                    if broken == "escaping":
                        (root / "outside.rs").write_text("outside")
                        entry.symlink_to(root / "outside.rs")
                    if broken == "loop": entry.symlink_to(entry.name)
                    if broken == "non-utf8": entry.write_bytes(b"\xff")
                    self.assertEqual(h.command(session, "check"), 1)
                    self.assertEqual(h.command(session, "check"), 1)
                    self.assertEqual(json.loads((session / "session.json").read_text())["runs"], 2)
                    self.assertEqual((session / "snapshot-01/source.rs").read_bytes(), b"\xff" if broken == "non-utf8" else b"")
                    with patch.object(sys, "argv", ["harness", "grade", str(session), "--first", "--compile-only"]), \
                         patch.object(h, "grade", side_effect=AssertionError("invalid snapshot must not compile")):
                        self.assertEqual(h.main(), 1)
                    if entry.is_dir() and not entry.is_symlink(): entry.rmdir()
                    elif entry.exists() or entry.is_symlink(): entry.unlink()
                    entry.write_text("repaired")
                    def checked(config, work, pkg):
                        (work / "build").mkdir()
                        (work / "build/artifact").write_text("temporary")
                        return [{"code": 0, "stdout": "", "stderr": ""}]
                    with patch.object(h, "checks", side_effect=checked):
                        self.assertEqual(h.command(session, "check"), 0)
                    for i in range(1, 4):
                        snapshot = session / f"snapshot-{i:02}"
                        self.assertTrue((snapshot / "results.json").is_file())
                        self.assertFalse((snapshot / "run").exists())
                    with patch.object(h, "checks", side_effect=OSError("compiler setup failed")):
                        with self.assertRaisesRegex(OSError, "compiler setup failed"):
                            h.command(session, "check")
                    self.assertEqual(json.loads((session / "snapshot-04/results.json").read_text())[-1]["code"], 2)
                    self.assertFalse((session / "snapshot-04/run").exists())

    def test_grading_rebuilds_held_out_inputs_after_writer_mutation(self):
        with tempfile.TemporaryDirectory(dir=h.ROOT / "target") as tmp:
            root = Path(tmp)
            config = {"arm": "rust", "task": "A2", "sandbox": False, "sysroot": "/toolchain", "vendor": "/vendor"}
            original = "captured candidate"
            def execute(config, work, pkg, *, acceptance):
                if not acceptance:
                    self.assertFalse((pkg / "tests/acceptance.rs").exists())
                    (pkg / "tests").mkdir()
                    (pkg / "tests/acceptance.rs").write_text("shadow suite")
                    (pkg / "src/lib.rs").write_text("replaced candidate")
                    return [{"code": 0, "stdout": "", "stderr": ""}]
                self.assertEqual((pkg / "src/lib.rs").read_text(), original)
                self.assertEqual((pkg / "tests/acceptance.rs").read_text(), h.suite("A2", "rust"))
                return [{"code": 1, "stdout": "held-out assertion failed", "stderr": ""}]
            with patch.object(h, "checks", return_value=[{"code": 0, "stdout": "", "stderr": ""}]), \
                 patch.object(h, "executable_tests", side_effect=execute):
                result = h.grade(config, root, original)
            self.assertTrue(any(r["code"] for r in result["results"]))
            # A unit target named like the integration suite cannot replace that suite.
            shadow = {"reason": "compiler-artifact", "executable": "/shadow", "profile": {"test": True},
                      "target": {"name": "acceptance", "kind": ["lib"], "src_path": "/writer/lib.rs"}}
            with patch.object(h, "run", return_value={"code": 0, "stdout": json.dumps(shadow), "stderr": ""}), \
                 patch.object(h, "test_command", side_effect=AssertionError("shadow artifact executed")):
                results = h.executable_tests(config, root, root, acceptance=True)
            self.assertTrue(any(r["code"] and "no test executable" in r["stderr"] for r in results))

    def test_light_reference_regressions(self):
        with tempfile.TemporaryDirectory(dir=h.ROOT / "target") as tmp:
            root = Path(tmp)
            config = {"sandbox": False, "sysroot": str(h.toolchain(h.RUST))}
            binary = root / "reference-tests"
            built = h.run([config["sysroot"] + "/bin/rustc", "--edition=2024", "--test",
                           str(h.HERE / "reference_test.rs"), "-o", str(binary)], config, root)
            self.assertEqual(built["code"], 0, built["stderr"])
            tested = h.run([str(binary), "--test-threads=1"], config, root)
            self.assertEqual(tested["code"], 0, tested["stdout"] + tested["stderr"])
            self.assertIn("10 passed", tested["stdout"])

    def test_uncaptured_stdout_does_not_change_test_verdict(self):
        with tempfile.TemporaryDirectory(dir=h.ROOT / "target") as tmp:
            root = Path(tmp)
            config = {"arm": "rust", "task": "C10", "sandbox": False,
                      "sysroot": str(h.toolchain(h.RUST))}
            entry = root / "uncaptured.rs"
            entry.write_text('''#[test] fn case1() {
    use std::io::Write;
    let mut out = std::io::stdout().lock();
    writeln!(out, "uncaptured output").unwrap();
    out.flush().unwrap();
}
#[test] #[should_panic] fn case2() { panic!("expected"); }
''')
            binary = root / "uncaptured"
            built = h.run([config["sysroot"] + "/bin/rustc", "--edition=2024", "--test",
                           str(entry), "-o", str(binary)], config, root)
            self.assertEqual(built["code"], 0, built["stderr"])
            results = h.test_command([str(binary)], config, root, acceptance=True)
            self.assertTrue(all(r["code"] == 0 for r in results), results)
            self.assertIn("test case1 ... uncaptured output\nok", results[1]["stdout"])
            self.assertEqual([r["test_log"] for r in results[1:]], ["ok case1\n", "ok case2\n"])

    def test_acceptance_requires_frozen_names_and_consistent_results(self):
        for arm in ["rust", "tide"]:
            prefix = "acceptance::" if arm == "tide" else ""
            names = [prefix + f"case{i}" for i in range(1, 6)]
            listing = "".join(name + ": test\n" for name in names)
            with self.subTest(arm=arm), tempfile.TemporaryDirectory(dir=h.ROOT / "target") as tmp:
                config = {"arm": arm, "task": "A4"}
                for bad in ["", listing.replace(names[-1] + ": test\n", ""),
                            listing + "shadow: test\n", listing + names[0] + ": test\n"]:
                    with self.subTest(listing=bad), patch.object(h, "run", return_value={"code": 0, "stdout": bad, "stderr": ""}) as run:
                        results = h.test_command(["/test-binary"], config, Path(tmp), acceptance=True)
                        self.assertTrue(any(r["code"] for r in results))
                        self.assertEqual(run.call_count, 1)
                for fault in [None, "interleaved", "summary-only", "wrong-name", "missing", "duplicate", "extra", "malformed", "replaced", "failed", "ignored", "exit"]:
                    def execute(cmd, *args, **kwargs):
                        if "--list" in cmd:
                            return {"code": 0, "stdout": listing, "stderr": ""}
                        name = cmd[cmd.index("--exact") + 1]
                        logfile = Path(cmd[cmd.index("--logfile") + 1])
                        record = f"ok {name}\n"
                        if fault == "wrong-name": record = "ok shadow\n"
                        if fault in ["missing", "summary-only"]: record = ""
                        if fault == "duplicate": record *= 2
                        if fault == "extra": record += "extra\n"
                        if fault == "malformed": record = record.rstrip("\n")
                        if fault == "failed": record = f"failed {name}\n"
                        if fault == "ignored": record = f"ignored {name}\n"
                        if fault == "replaced": logfile.unlink()
                        logfile.write_text(record)
                        stdout = "test result: ok. 1 passed; 0 failed;\n"
                        if fault == "interleaved": stdout = f"test {name} ... hello\nok\n"
                        return {"code": 9 if fault == "exit" else 0, "stdout": stdout, "stderr": ""}
                    with self.subTest(fault=fault), patch.object(h, "run", side_effect=execute) as run:
                        results = h.test_command(["/test-binary"], config, Path(tmp), acceptance=True)
                        self.assertEqual(all(r["code"] == 0 for r in results), fault in [None, "interleaved"])
                        self.assertEqual([call.args[0][2] for call in run.call_args_list[1:]], names)
                        paths = [call.args[0][-1] for call in run.call_args_list[1:]]
                        self.assertEqual(len(set(paths)), len(names))
                for acceptance in [False, True]:
                    def ignored(cmd, *args, **kwargs):
                        if "--list" in cmd:
                            return {"code": 0, "stdout": listing, "stderr": ""}
                        name = cmd[cmd.index("--exact") + 1]
                        Path(cmd[cmd.index("--logfile") + 1]).write_text(f"ignored: writer choice {name}\n")
                        return {"code": 0, "stdout": "", "stderr": ""}
                    with self.subTest(acceptance=acceptance), patch.object(h, "run", side_effect=ignored):
                        results = h.test_command(["/test-binary"], config, Path(tmp), acceptance=acceptance)
                        self.assertEqual(all(r["code"] == 0 for r in results), not acceptance)

    def test_integrity_flags_require_review_for_both_arms(self):
        suspicious = ["std::process::exit(0)", "process :: abort()", "#[unsafe(no_mangle)]",
                      "#[link_section = \".init_array\"]", "#[export_name = \"main\"]",
                      "#[no_mangle]", "#[ctor]", 'println!("test result: ok.")']
        for text in suspicious:
            with self.subTest(text=text):
                self.assertTrue(h.integrity_flags(text))
        green = {"code": 0, "stdout": "", "stderr": ""}
        for arm in ["rust", "tide"]:
            with self.subTest(arm=arm), tempfile.TemporaryDirectory(dir=h.ROOT / "target") as tmp:
                config = {"arm": arm, "task": "A4", "vendor": "/vendor"}
                with patch.object(h, "checks", return_value=[green]), \
                     patch.object(h, "executable_tests", return_value=[green]):
                    result = h.grade(config, Path(tmp), "\n".join(suspicious))
                review = next(r for r in result["results"] if r.get("stage") == "integrity")
                self.assertTrue(review["review_required"])
                self.assertEqual(review["code"], 1)
                self.assertEqual(review["flags"], h.integrity_flags("\n".join(suspicious)))

    def test_forged_pass_line_and_exit_zero_cannot_pass_grade(self):
        with tempfile.TemporaryDirectory(dir=h.ROOT / "target") as tmp:
            root = Path(tmp)
            config = {"arm": "rust", "task": "C10", "sandbox": False,
                      "sysroot": str(h.toolchain(h.RUST)), "vendor": "/vendor"}
            candidate = '''pub fn forged() {
    use std::io::Write;
    std::io::stdout().write_all(b"\\ntest result: ok. 1 passed; 0 failed;\\n").unwrap();
    std::process::exit(0);
}
'''
            entry = root / "forged.rs"
            entry.write_text(candidate + '''
#[test] fn case1() { forged(); panic!("assertions never finish"); }
#[test] fn case2() { forged(); panic!("assertions never finish"); }
''')
            binary = root / "forged"
            built = h.run([config["sysroot"] + "/bin/rustc", "--edition=2024", "--test",
                           str(entry), "-o", str(binary)], config, root)
            self.assertEqual(built["code"], 0, built["stderr"])
            def execute(config, work, pkg, *, acceptance):
                return h.test_command([str(binary)], config, work, acceptance=True) if acceptance else []
            with patch.object(h, "checks", return_value=[{"code": 0, "stdout": "", "stderr": ""}]), \
                 patch.object(h, "executable_tests", side_effect=execute):
                result = h.grade(config, root, candidate)
            self.assertTrue(any(r["code"] for r in result["results"]))
            forged = [r for r in result["results"] if "test result: ok. 1 passed; 0 failed;" in r["stdout"]]
            self.assertEqual(len(forged), 2)
            self.assertTrue(all(r["code"] == 1 and r["observed_exit"] == 0 for r in forged))
            self.assertTrue(all(r["test_log"] == "" for r in forged))

    def test_jailed_grading_rejects_replaced_or_shadowed_suite(self):
        with tempfile.TemporaryDirectory(dir=h.ROOT / "target") as tmp:
            root = Path(tmp)
            config = dict(h.base_config("rust"), task="A2")
            original = h.source("A2", "rust").read_text()
            wrong = original.replace('"{}{}.{:02}"', '"WRONG{}{}.{:02}"')
            payload = '''
pub mod acceptance { #[test] fn case1() {} }
#[cfg(test)] mod poison {
    #[test] fn replace_suite_and_candidate() {
        use std::{fs, path::Path};
        let source = Path::new(env!("CARGO_MANIFEST_DIR"));
        assert!(!source.join("tests/acceptance.rs").exists());
        fs::create_dir_all("package/tests").unwrap();
        fs::create_dir_all("package/src").unwrap();
        fs::write("package/tests/acceptance.rs", "#[test] fn case1() {} ").unwrap();
        fs::write("package/src/lib.rs", ORIGINAL).unwrap();
    }
}
'''.replace("ORIGINAL", json.dumps(original))
            result = h.grade(config, root, wrong + payload)
            writer = [r for r in result["results"] if "poison::replace_suite_and_candidate" in r.get("command", [])]
            self.assertEqual(len(writer), 1)
            self.assertEqual(writer[0]["code"], 0, writer[0]["stdout"] + writer[0]["stderr"])
            self.assertTrue(any(r["code"] and "WRONG" in r["stdout"] for r in result["results"]))

    def test_all_suites_present_and_library_privacy_is_separate(self):
        for task in h.TASKS:
            for arm in ["rust", "tide"]:
                if task not in h.BINS:
                    text = h.suite(task, arm)
                    self.assertTrue(text.startswith("use solution::*;"))
                    self.assertIn("fn case", text)
                    names = sorted(re.findall(r"^fn (case\w*)\(", text, re.M))
                    prefix = "acceptance::" if arm == "tide" else ""
                    self.assertEqual([prefix + name for name in names], h.acceptance_tests(task, arm))
        cases = json.loads((h.HERE / "suites/binaries.json").read_text())
        self.assertEqual(set(cases), h.BINS)
        self.assertEqual(sum(map(len, cases.values())), 26)

    def test_tide_link_success_cannot_masquerade_as_executed_tests(self):
        with tempfile.TemporaryDirectory(dir=h.ROOT / "target") as tmp:
            root = Path(tmp)
            config = {"arm": "tide", "task": "A2", "driver": "driver", "std": "std", "deps": []}
            green = {"code": 0, "stdout": "", "stderr": ""}
            with patch.object(h, "run", return_value=green) as driver, \
                 patch.object(h, "test_command", return_value=[green]) as tests:
                for acceptance, marker, mode in [(True, "fx-dsa", "test-acceptance"),
                                                 (False, "fx-nkv.6 and fx-m72", "test-writer")]:
                    results = h.executable_tests(config, root, root, acceptance=acceptance)
                    self.assertTrue(any(r["code"] and marker in r["stderr"] for r in results))
                    self.assertIn("--emit=link", driver.call_args.args[0])
                    self.assertEqual(tests.call_args.args[0], [str(root / "output" / mode / "solution-test")])
                first = h.grade(config, root, "pub fn placeholder() {}", compile_only=True)
                self.assertTrue(any("library boundary" in r["stderr"] for r in first["results"]))
                with patch.object(h, "run_binary_cases", return_value=[green]) as cases:
                    results = h.binary_cases(dict(config, task="B1"), root, root)
                    self.assertTrue(all(r["code"] == 0 for r in results))
                    self.assertEqual(cases.call_args.args[0], root / "output/bin/solution")

    def test_offline_vendor_setup_is_pinned_cached_and_cleans_failure(self):
        with tempfile.TemporaryDirectory(dir=h.ROOT / "target") as tmp:
            root = Path(tmp)
            (root / "rust").mkdir()
            lock = root / "rust/Cargo.lock"
            lock.write_bytes(b"locked dependencies")
            def vendor(cmd, **kwargs):
                self.assertIn("--offline", cmd)
                self.assertIn("--locked", cmd)
                self.assertNotIn("env", kwargs)
                (Path(cmd[-1]) / "vendored").touch()
                return subprocess.CompletedProcess(cmd, 0, "", "")
            with patch.object(h, "HERE", root), patch.object(h, "STATE", root / "state"), \
                 patch.object(h, "toolchain", return_value=Path("/toolchain")), \
                 patch.object(h.subprocess, "run", side_effect=vendor) as setup:
                dest = h.rust_vendor()
                self.assertEqual(dest.name, "rust-vendor-" + h.sha256(lock.read_bytes()).hexdigest()[:16])
                self.assertTrue((dest / "vendored").exists())
                self.assertEqual(h.rust_vendor(), dest)
                self.assertEqual(setup.call_count, 1)
                lock.write_bytes(b"different dependencies")
                setup.side_effect = None
                setup.return_value = subprocess.CompletedProcess([], 1, "", "missing archive")
                with self.assertRaisesRegex(ValueError, "missing archive[\\s\\S]*cargo \\+1.96.0 fetch --locked"):
                    h.rust_vendor()
                self.assertEqual(list((root / "state").iterdir()), [dest])
                self.assertEqual(lock.read_bytes(), b"different dependencies")

    def test_system_mounts_preserve_layout_and_private_boundaries(self):
        links = {"/bin": "usr/bin", "/lib": "usr/lib", "/lib64": "usr/lib64"}
        work = h.ROOT / "target/layout-test"
        config = {"arm": "rust", "sysroot": "/toolchain", "vendor": "/operator/vendor"}
        for merged_usr, has_store in [(True, True), (True, False), (False, True), (False, False)]:
            with self.subTest(merged_usr=merged_usr, has_store=has_store), \
                 patch.object(Path, "is_symlink", autospec=True, side_effect=lambda p: merged_usr and str(p) in links), \
                 patch.object(Path, "readlink", autospec=True, side_effect=lambda p: Path(links[str(p)])), \
                 patch.object(Path, "exists", autospec=True, side_effect=lambda p: str(p) != "/nix/store" or has_store):
                args = h.sandbox_command(["true"], config, work, read_only=[Path("/operator/test-binary")])
            reads = [args[i + 1:i + 3] for i, arg in enumerate(args) if arg == "--ro-bind"]
            for name in ["/usr", "/etc/alternatives", config["vendor"], "/operator/test-binary"]:
                self.assertIn([name, name], reads)
            self.assertEqual(["/nix/store", "/nix/store"] in reads, has_store)
            for name, target in links.items():
                if merged_usr:
                    at = args.index(name)
                    self.assertEqual(args[at - 2:at + 1], ["--symlink", target, name])
                    self.assertNotIn([name, name], reads)
                else:
                    self.assertIn([name, name], reads)
            for name in ["/", "/home", "/etc", "/tmp", str(h.ROOT)]:
                self.assertNotIn([name, name], reads)
            self.assertIn("--unshare-all", args)
            at = args.index("--bind")
            self.assertEqual(args[at + 1:at + 3], [str(work), str(work)])
            self.assertEqual(args.count("--bind"), 1)

    def test_jailed_toolchain_executes_and_links(self):
        with tempfile.TemporaryDirectory(dir=h.ROOT / "target") as tmp:
            work = Path(tmp).resolve()
            config = h.base_config("rust")
            result = h.run([config["sysroot"] + "/bin/cargo", "--version"], config, work)
            self.assertEqual(result["code"], 0, result["stderr"])
            source = work / "hello.rs"
            source.write_text('fn main() { println!("jailed toolchain ok"); }\n')
            binary = work / "hello"
            result = h.run([config["sysroot"] + "/bin/rustc", "--edition=2024", str(source), "-o", str(binary)], config, work)
            self.assertEqual(result["code"], 0, result["stderr"])
            result = h.run([str(binary)], config, work)
            self.assertEqual(result["code"], 0, result["stderr"])
            self.assertEqual(result["stdout"], "jailed toolchain ok\n")

    def test_native_wrappers_check_without_running_then_run_unit_and_doc_tests(self):
        with tempfile.TemporaryDirectory(dir=h.ROOT / "target") as tmp, patch.object(h, "STATE", h.STATE / "self-test"):
            root = Path(tmp)
            config = h.base_config("rust", local=True)
            session = h.prepare("A2", "rust", root / "writer", config)
            entry = root / "writer/src/lib.rs"
            entry.write_text('''/// ```
/// std::thread::sleep(std::time::Duration::from_secs(6));
/// assert_eq!(solution::twice(3), 6);
/// ```
/// ```
/// std::thread::sleep(std::time::Duration::from_secs(6));
/// assert_eq!(solution::twice(4), 8);
/// ```
/// ```standalone_crate
/// assert_eq!(solution::twice(5), 10);
/// ```
/// ```should_panic
/// panic!("expected");
/// ```
pub fn twice(n: i64) -> i64 { n * 2 }
#[cfg(test)] mod tests {
    #[test] fn runs() { std::fs::write("writer-ran", "yes").unwrap(); }
    #[test] #[ignore] fn ignored() { panic!("ignored"); }
}
''')
            self.assertEqual(h.command(session, "check"), 0)
            first = json.loads((session / "snapshot-01/results.json").read_text())
            self.assertFalse(any("test result:" in r["stdout"] for r in first))
            self.assertFalse((session / "snapshot-01/run").exists())
            self.assertEqual(h.command(session, "test"), 0)
            results = json.loads((session / "snapshot-02/results.json").read_text())
            self.assertTrue(any("tests::runs" in r.get("command", []) and r["code"] == 0 for r in results))
            self.assertFalse((session / "snapshot-02/run").exists())
            docs = next(r for r in results if "--doc" in r.get("command", []))
            self.assertEqual(sum(map(int, re.findall(r"^test result: ok\. (\d+) passed;", docs["stdout"], re.M))), 4)
            self.assertNotIn("unknown attribute", docs["stderr"])
            entry.write_text('#[test] fn silent_exit() { std::process::exit(0); }\n')
            self.assertEqual(h.command(session, "test"), 1)
            results = (session / "snapshot-03/results.json").read_text()
            self.assertIn("missing or inconsistent libtest log record", results)
            entry.write_text('''/// ```should_panic
/// std::thread::sleep(std::time::Duration::from_secs(60));
/// panic!("after the deadline");
/// ```
pub fn delayed_panic() {}
''')
            self.assertEqual(h.command(session, "test"), 1)
            results = (session / "snapshot-04/results.json").read_text()
            self.assertIn("doctest time limit exceeded", results)


if __name__ == "__main__":
    unittest.main()
