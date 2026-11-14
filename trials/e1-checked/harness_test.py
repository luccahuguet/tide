#!/usr/bin/env python3
"""Small contract checks for rendering, structure, budgets, and child timeouts."""
import io
import os
import json
from pathlib import Path
import re
import signal
import subprocess
import sys
import tempfile
import threading
import time
import unittest
from unittest.mock import patch

import harness as h
import broker as b
import coordinator as c
import gate_checks as g
from standin import Broker


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
                self.assertEqual(h.command(session, "check"), 3)
                state["runs"] = 0
                state["started"] = time.time() - 2401
                (session / "session.json").write_text(json.dumps(state))
                self.assertEqual(h.command(session, "check"), 3)

    def test_timeout_and_clean_environment(self):
        with tempfile.TemporaryDirectory(dir=h.ROOT / "target") as tmp:
            root = Path(tmp)
            config = {"sandbox": False, "sysroot": "/toolchain"}
            result = h.run([sys.executable, "-c", "import time; time.sleep(10)"], config, root, timeout=0.05)
            self.assertEqual(result["code"], 124)
            result = h.run([sys.executable, "-c", "import os,sys; print(os.environ.get('E1_SECRET')); print(sys.stdin.read())"], config, root)
            self.assertEqual(result["code"], 0)
            self.assertEqual(result["stdout"], "None\n\n")
            with patch.dict(h.os.environ, {"CARGO_BUILD_JOBS": "6"}):
                self.assertEqual(h.environment(config, root)["CARGO_BUILD_JOBS"], "4")
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
            names = [f"case{i}" for i in range(1, 6)]
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
                    self.assertEqual(names, h.acceptance_tests(task))
        cases = json.loads((h.HERE / "suites/binaries.json").read_text())
        self.assertEqual(set(cases), h.BINS)
        self.assertEqual(sum(map(len, cases.values())), 26)

    def test_tide_link_success_cannot_masquerade_as_executed_tests(self):
        with tempfile.TemporaryDirectory(dir=h.ROOT / "target") as tmp:
            root = Path(tmp)
            config = {"arm": "tide", "task": "A2", "driver": "driver", "std": "std", "deps": ["/operator"]}
            green = {"code": 0, "stdout": "", "stderr": ""}
            with patch.object(h, "run", return_value=green) as driver, \
                 patch.object(h, "test_command", return_value=[green]) as tests:
                results = h.executable_tests(config, root, root, acceptance=False)
                self.assertTrue(all(r["code"] == 0 for r in results))
                self.assertIn("--emit=link", driver.call_args.args[0])
                self.assertEqual(tests.call_args.args[0], [str(root / "output/test-writer/solution-test")])
                with patch.object(h, "run_binary_cases", return_value=[green]) as cases:
                    results = h.binary_cases(dict(config, task="B1"), root, root)
                    self.assertTrue(all(r["code"] == 0 for r in results))
                    self.assertEqual(cases.call_args.args[0], root / "output/bin/solution")

    def test_generated_module_segment_is_filtered_only_from_acceptance(self):
        green = {"code": 0, "stdout": "", "stderr": ""}
        generated = "nested::__tide_contract_tests::generated"
        for acceptance in [False, True]:
            with self.subTest(acceptance=acceptance), tempfile.TemporaryDirectory(dir=h.ROOT / "target") as tmp:
                listing = dict(green, stdout="case1: test\ncase2: test\n" + generated + ": test\n")
                seen = []
                def run(cmd, *_args, **_kwargs):
                    if "--list" in cmd: return listing
                    test = cmd[cmd.index("--exact") + 1]
                    seen.append(test)
                    Path(cmd[cmd.index("--logfile") + 1]).write_text("ok " + test + "\n")
                    return dict(green)
                with patch.object(h, "run", side_effect=run):
                    result = h.test_command(["/binary"], {"task": "C10"}, Path(tmp), acceptance=acceptance)
                self.assertTrue(all(r["code"] == 0 for r in result), result)
                self.assertEqual(seen, ["case1", "case2"] + ([] if acceptance else [generated]))
        # A substring is not the reserved module segment.
        with tempfile.TemporaryDirectory(dir=h.ROOT / "target") as tmp:
            listing = dict(green, stdout="case1: test\ncase2: test\n__tide_contract_tests_extra::case: test\n")
            with patch.object(h, "run", return_value=listing):
                result = h.test_command(["/binary"], {"task": "C10"}, Path(tmp), acceptance=True)
            self.assertEqual(result[0]["code"], 1)

    def test_tide_writer_runs_every_test_form_and_reports_unchecked(self):
        with tempfile.TemporaryDirectory(dir=h.ROOT / "target") as tmp:
            root = Path(tmp)
            config = dict(h.base_config("tide", local=True), task="A2")
            independent = (h.ROOT / "conformance/cases/contracts/generated_tests.rs").read_text()
            independent += """
#[test] fn property(n: u8) { assert!(n <= u8::MAX); }
#[sim_test(seeds = 2)] fn simulation(sim: std::sim::SimRoot) { let _ = sim.clock.now(); }
"""
            pkg = h.package(root, config, independent)
            results = h.executable_tests(config, root, pkg, acceptance=False)
            self.assertEqual(results[0]["code"], 0, results[0]["stderr"])
            executions = {r["command"][r["command"].index("--exact") + 1]: r
                          for r in results if "--exact" in r.get("command", [])}
            for name in ["plain", "property", "simulation"]:
                self.assertEqual(executions[name]["code"], 0, executions[name])
            rebound = next(r for name, r in executions.items() if name.startswith("__tide_contract_tests::rebound_"))
            self.assertNotEqual(rebound["code"], 0)
            self.assertIn("ensures violated:", rebound["stdout"] + rebound["stderr"])
            allowed = next(r for name, r in executions.items() if name.startswith("__tide_contract_tests::allowed_"))
            self.assertEqual(allowed["code"], 0, allowed)
            self.assertIn("unchecked (vacuous, allowed:", allowed["stdout"] + allowed["stderr"])
            skipped = next(r for name, r in executions.items() if name.startswith("__tide_contract_tests::handle_arg_"))
            self.assertEqual(skipped["code"], 0, skipped)
            self.assertIn("unchecked:", skipped["test_log"])

    def test_tide_library_boundary_compiles_separate_sources_before_execution(self):
        for first, code in [(True, 0), (False, 0), (False, 1)]:
            with self.subTest(first=first, code=code), tempfile.TemporaryDirectory(dir=h.ROOT / "target") as tmp:
                root = Path(tmp)
                config = {"arm": "tide", "task": "A2", "driver": "driver", "std": "std", "deps": ["/operator"]}
                candidate = "pub fn placeholder() {}"
                def compile(cmd, *_args):
                    if not any(arg.startswith("--tide-test-library=") for arg in cmd):
                        return {"code": 0, "stdout": "", "stderr": ""}
                    library = root / "acceptance/package/src/lib.rs"
                    entry = root / "acceptance/suite/src/lib.rs"
                    self.assertEqual(library.read_text(), candidate)
                    self.assertEqual(entry.read_text(), "use tide_candidate as solution;\n" + h.suite("A2", "tide"))
                    # Generated code must admit under the driver: `extern crate` is FX0912.
                    self.assertNotIn("extern crate", entry.read_text())
                    self.assertEqual(cmd[-1], str(entry))
                    self.assertIn("--tide-test-library=" + str(library), cmd)
                    self.assertEqual([arg for arg in cmd if arg.startswith("--tide-dep-dir=")], ["--tide-dep-dir=/operator"])
                    self.assertIn("--emit=" + ("metadata" if first else "link"), cmd)
                    return {"code": code, "stdout": "", "stderr": ""}
                with patch.object(h, "run", side_effect=compile), \
                     patch.object(h, "test_command", return_value=[]) as tests:
                    result = h.grade(config, root, candidate, compile_only=first)
                    self.assertEqual([r["code"] for r in result["results"] if r.get("stage") == "L"], [code])
                    acceptance_calls = [call for call in tests.call_args_list if call.kwargs.get("acceptance")]
                    if first or code:
                        self.assertEqual(acceptance_calls, [])
                    else:
                        self.assertEqual(len(acceptance_calls), 1)
                        self.assertEqual(acceptance_calls[0].args[0], [str(root / "acceptance/output/test-acceptance/acceptance-test")])

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

    def test_tide_offline_vendor_setup_is_pinned_cached_and_cleans_failure(self):
        with tempfile.TemporaryDirectory(dir=h.ROOT / "target") as tmp:
            root = Path(tmp)
            lock = root / "Cargo.lock"
            lock.write_bytes(b"locked dependencies")
            def vendor(cmd, **kwargs):
                self.assertIn("--offline", cmd)
                self.assertIn("--locked", cmd)
                self.assertEqual(cmd[0], "/toolchain/bin/cargo")
                self.assertEqual(cmd[cmd.index("--manifest-path") + 1], str(root / "Cargo.toml"))
                self.assertNotIn("env", kwargs)
                (Path(cmd[-1]) / "vendored").touch()
                return subprocess.CompletedProcess(cmd, 0, "", "")
            with patch.object(h, "ROOT", root), patch.object(h, "STATE", root / "state"), \
                 patch.object(h, "toolchain", return_value=Path("/toolchain")), \
                 patch.object(h.subprocess, "run", side_effect=vendor) as setup:
                dest = h.tide_vendor()
                self.assertEqual(dest.name, "tide-vendor-" + h.sha256(lock.read_bytes()).hexdigest()[:16])
                self.assertTrue((dest / "vendored").exists())
                self.assertEqual(h.tide_vendor(), dest)
                self.assertEqual(setup.call_count, 1)
                lock.write_bytes(b"different dependencies")
                setup.side_effect = None
                setup.return_value = subprocess.CompletedProcess([], 1, "", "missing archive")
                with self.assertRaisesRegex(ValueError, "missing archive[\\s\\S]*cargo \\+nightly-2026-08-09 fetch --locked"):
                    h.tide_vendor()
                self.assertEqual(list((root / "state").iterdir()), [dest])
                self.assertEqual(lock.read_bytes(), b"different dependencies")

    def test_system_mounts_preserve_layout_and_private_boundaries(self):
        links = {"/bin": "usr/bin", "/lib": "usr/lib", "/lib64": "usr/lib64"}
        work = h.ROOT / "target/layout-test"
        closure = ["/nix/store/" + "a" * 32 + "-glibc", "/nix/store/" + "b" * 32 + "-zlib"]
        config = {"arm": "rust", "sysroot": "/toolchain", "vendor": "/operator/vendor"}
        for merged_usr, has_store in [(True, True), (True, False), (False, True), (False, False)]:
            with self.subTest(merged_usr=merged_usr, has_store=has_store), \
                 patch.object(Path, "is_symlink", autospec=True, side_effect=lambda p: merged_usr and str(p) in links), \
                 patch.object(Path, "readlink", autospec=True, side_effect=lambda p: Path(links[str(p)])), \
                 patch.object(Path, "exists", autospec=True, side_effect=lambda p: str(p) != "/nix/store" or has_store):
                args = h.sandbox_command(["true"], dict(config, store_paths=closure if has_store else []), work,
                                         read_only=[Path("/operator/test-binary")])
            reads = [args[i + 1:i + 3] for i, arg in enumerate(args) if arg == "--ro-bind"]
            for name in ["/usr", "/etc/alternatives", config["vendor"], "/operator/test-binary"]:
                self.assertIn([name, name], reads)
            self.assertNotIn(["/nix/store", "/nix/store"], reads)
            self.assertEqual(all([p, p] in reads for p in closure), has_store)
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
            config["without_store"] = bool(h.os.environ.get("E1_WITHOUT_STORE_TEST"))
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

    def test_jailed_runtime_executable_under_tmp_survives_tmpfs(self):
        with tempfile.TemporaryDirectory(dir="/tmp") as tmp:
            root = Path(tmp)
            config = h.base_config("rust")
            source = root / "control.rs"
            source.write_text('fn main() { println!("external tmp runtime ok"); }\n')
            (root / "output").mkdir()
            binary = root / "output/control"
            built = h.run([config["sysroot"] + "/bin/rustc", str(source), "-o", str(binary)], config, root)
            self.assertEqual(built["code"], 0, built["stderr"])
            scratch = root / "scratch"
            scratch.mkdir()
            result = h.run([str(binary)], config, scratch, read_only=[binary])
            self.assertEqual(result["code"], 0, result["stderr"])
            self.assertEqual(result["stdout"], "external tmp runtime ok\n")

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


class GateTests(unittest.TestCase):
    def test_dry_flag_cannot_launch_existing_gate_state(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            g.save(root / "state.json", {"dry_run": False, "model": g.MODEL, "cli": "/pinned", "jobs": 1})
            with patch.object(sys, "argv", ["coordinator", "run", str(root), "--dry-run"]), \
                 patch.object(c, "batch") as batch:
                with self.assertRaisesRegex(ValueError, "cannot turn a gate"):
                    c.main()
                batch.assert_not_called()

    def test_dirty_identity_and_changed_cli_stop_before_next_writer(self):
        with patch.object(g, "output", return_value=" M crates/control.rs"):
            with self.assertRaisesRegex(ValueError, "clean"):
                g.clean()
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            state = {"dry_run": False, "order": [["A2", "rust", 1]], "jobs": 1,
                     "configs": {"rust": {}}, "model": g.MODEL, "cli": "unused", "seconds": 2400,
                     "identity": {"cheap": {"cli_version": "pinned"}}}
            with patch.object(c, "prerequisites"), patch.object(g, "cheap", return_value={"cli_version": "changed"}), \
                 patch.object(h, "prepare") as prepared:
                with self.assertRaisesRegex(RuntimeError, "identity changed"):
                    c.batch(root, state, 1)
                prepared.assert_not_called()

    def test_canary_rejects_leaks_missing_probes_and_broken_controls(self):
        class Socket:
            def __init__(self, *_args): pass
            def __enter__(self): return self
            def __exit__(self, *_args): pass
            def connect(self, *_args): pass
            def bind(self, *_args): pass
            def listen(self, *_args): pass
            def getsockname(self): return ("192.0.2.1", 1234)
        for broken in [None, "repository", "network", "missing", "positive", "negative", "tmpfs_size"]:
            def run(cmd, config, work, **kwargs):
                spec = json.loads(cmd[-1])
                checks = {name: not config["sandbox"] for name in
                          set(spec["paths"]) | set(spec["addresses"]) | {"environment", "dns"}}
                if config["sandbox"] and broken in {"repository", "network"}: checks[broken] = True
                if config["sandbox"] and broken == "missing": checks.pop("reference")
                if not config["sandbox"] and broken == "negative": checks["network"] = False
                raw = {"sentinel": "E1-CANARY", "checks": checks, "positive": broken != "positive", "tmpfs": True,
                       "tmpfs_bytes": h.TMPFS_SIZE if broken != "tmpfs_size" else 2 * h.TMPFS_SIZE}
                return {"code": 0, "stdout": json.dumps(raw), "stderr": ""}
            with self.subTest(broken=broken), tempfile.TemporaryDirectory() as tmp, \
                 patch.object(h, "STATE", Path(tmp)), patch.object(h, "run", side_effect=run), \
                 patch.object(g.socket, "socket", Socket), patch.object(g, "canary_key", return_value={"revision": 1}):
                config = {"arm": "rust", "sysroot": "/toolchain", "sandbox": True}
                if broken:
                    with self.assertRaisesRegex(ValueError, "canary failed|control failed"):
                        g.canary("rust", config)
                else:
                    record = g.canary("rust", config)
                    self.assertEqual(len(record["records"]), 3)
                    # The saved record carries each shape's negative-control probe output.
                    self.assertTrue(all(r["negative_control"]["positive"] for r in json.loads(json.dumps(record))["records"]))

    def test_batch_persists_faults_and_recovers_interrupted_attempt(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            state = {"dry_run": True, "order": [["A2", "rust", 1]], "jobs": 1,
                     "configs": {"rust": {}}, "model": g.MODEL, "cli": "unused", "seconds": 2400}
            def prepare(task, arm, directory, config, *, session):
                directory.mkdir(parents=True)
                session.mkdir(parents=True)
                g.save(session / "session.json", {"directory": str(directory), "runs": 0})
            calls = []
            def writer(session, *_args, **kwargs):
                kwargs["on_started"]()
                calls.append(session.name)
                return {"fault": None, "end_reason": "done"}
            g.save(root / "slot-000.json", {"complete": False, "attempts": [
                {"fault": "host_interruption", "end_reason": "host_interruption"}]})
            first = root / "sessions/000-1"
            first.mkdir(parents=True)
            g.save(first / "launch.json", {})
            g.save(first / "process.json", {"boot": "old boot"})
            with patch.object(h, "prepare", side_effect=prepare), patch.object(c, "run_writer", side_effect=writer), \
                 patch.object(c, "freeze_final", return_value="hash"), \
                 patch.object(c, "freeze_and_grade", return_value={"first": {"exit": 1}, "final": {"exit": 1}}) as grades:
                c.batch(root, state, 1)
            row = json.loads((root / "slot-000.json").read_text())
            self.assertEqual(calls, ["000-2"])
            self.assertEqual(grades.call_count, 1)
            self.assertEqual(row["reruns"], 1)
            self.assertTrue(row["complete"])
            self.assertTrue(row["dry_run"])
            self.assertFalse(row["first_pass"])
            self.assertTrue((root / "sessions/000-2/trial.json").exists())

    def test_real_broker_relays_both_budget_refusals_without_compiling(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            g.save(root / "session.json", {"runs": 20, "started": time.time()})
            mcp = c.mcp_config(root)
            broker = Broker(mcp)
            try:
                self.assertEqual(broker.tool("check"), 3)
                g.save(root / "session.json", {"runs": 0, "started": time.time(),
                                              "started_monotonic": time.monotonic() - 2401})
                self.assertEqual(broker.tool("test"), 3)
            finally:
                broker.close()
                broker.proc.stdout.close()
            self.assertEqual([row["exit"] for row in c.jsonl(root / "broker.jsonl") if "exit" in row], [3, 3])
            self.assertEqual(list(root.glob("snapshot-*")), [])

    def test_setup_status_is_separate_from_candidate_exit_and_stderr(self):
        config = {"arm": "rust", "sandbox": True, "sysroot": "/toolchain", "vendor": "/vendor"}
        for started in [False, True]:
            class Process:
                returncode = 2
                def __init__(self, argv, **kwargs):
                    limit_dir = Path(argv[argv.index(str(h.HERE / "jail_limits.py")) + 2])
                    (limit_dir / "report.json").write_text(json.dumps({"max": 0, "oom": 0, "oom_kill": 0, "memory_peak": 1, "cpu_usage_usec": 1,
                        "memory_max": str(h.MEMORY_MAX), "memory_swap_max": "0"}))
                    if started:
                        h.os.write(kwargs["pass_fds"][0], b'{"exit-code": 2}\n')
                def communicate(self, *_args, **_kwargs):
                    return b"", b"bwrap: candidate-controlled output\n"
            with self.subTest(started=started), tempfile.TemporaryDirectory() as tmp, \
                 patch.object(h.subprocess, "Popen", Process):
                result = h.run(["/binary"], config, Path(tmp))
                self.assertEqual(result["infrastructure"], not started)
        with tempfile.TemporaryDirectory() as tmp, patch.object(h, "STATE", Path(tmp) / "state"):
            root = Path(tmp)
            session = h.prepare("A2", "rust", root / "writer", dict(config, sandbox=False))
            for infrastructure, expected in [(False, 1), (True, 2)]:
                with patch.object(h, "checks", return_value=[{"code": 2, "infrastructure": infrastructure, "stdout": "", "stderr": ""}]):
                    self.assertEqual(h.command(session, "check"), expected)

    def test_frozen_tuple_order(self):
        slots = c.order()
        self.assertEqual(len(set(slots)), 120)
        self.assertEqual(sum(arm == "rust" for _, arm, _ in slots), 60)
        self.assertEqual(slots[:4], [("C1", "rust", 2), ("B6", "rust", 1), ("A9", "tide", 1), ("A10", "tide", 1)])
        self.assertEqual(c.order_hash(slots), "9d0951c1c2301efa926f91a70f9bc1dfd9708015b6cbf1bc2c106ca818265c50")

    def test_closed_fault_list_and_single_immediate_rerun(self):
        for endings in [["init_failure", None], ["broker_crash", "cli_error"], [None]]:
            records, called = [], []
            def runner(attempt):
                called.append(attempt)
                return {"fault": endings[attempt - 1], "end_reason": endings[attempt - 1] or "budget_refusal"}
            result = c.run_slot(0, ["A2", "rust", 1], runner, records.append)
            self.assertEqual(len(result), len(endings))
            self.assertEqual(called, list(range(1, len(endings) + 1)))
            self.assertTrue(records[-1]["complete"])
            self.assertEqual(records[-1]["failed_infrastructure"], endings[-1] is not None)
            self.assertEqual(records[-1]["reruns"], len(endings) - 1)
        with self.assertRaisesRegex(ValueError, "unclassified"):
            c.run_slot(0, [], lambda _: {"fault": "wrong_output"}, lambda _: None)

    def test_init_and_log_audit_fail_closed(self):
        init = {"type": "system", "subtype": "init", "tools": sorted(g.TOOLS), "plugins": [],
                "mcp_servers": [{"name": "e1", "status": "connected"}], "model": g.MODEL}
        c.init_assertion(init, g.MODEL)
        for changed in [{"tools": sorted(g.TOOLS) + ["Bash"]}, {"plugins": ["x"]}, {"model": "other"},
                        {"mcp_servers": [{"name": "e1", "status": "failed"}]}, {"tools": []}]:
            with self.subTest(changed=changed), self.assertRaises(ValueError):
                c.init_assertion(dict(init, **changed), g.MODEL)
        with tempfile.TemporaryDirectory() as tmp:
            directory = Path(tmp)
            (directory / "link").symlink_to("/etc")
            def event(name, args):
                return {"type": "assistant", "message": {"model": g.MODEL, "usage": {"input_tokens": 4},
                        "content": [{"type": "tool_use", "id": "1", "name": name, "input": args}]}}
            for name, args in [("Bash", {}), ("Read", {"file_path": "/etc/passwd"}),
                               ("Read", {"file_path": "link/passwd"}), ("mcp__e1__check", {"session": "other"})]:
                self.assertFalse(c.audit([event(name, args)], directory, g.MODEL)["passed"])
            valid = c.audit([event("Read", {"file_path": "reference.md", "offset": 8, "limit": 10})], directory, g.MODEL)
            self.assertTrue(valid["passed"])
            self.assertEqual(valid["tokens"]["input_tokens"], 4)
            self.assertEqual(valid["reference_reads"][0]["offset"], 8)

    def test_broker_schema_refusal_and_output_cap(self):
        requests = [{"id": 1, "method": "tools/list"},
                    {"id": 2, "method": "tools/call", "params": {"name": "check", "arguments": {"session": "escape"}}},
                    {"id": 3, "method": "tools/call", "params": {"name": "test", "arguments": {}}}]
        class Process:
            returncode = 3
            def communicate(self, **kwargs): return b"x" * (b.CAP + 1), b"refusal"
        with tempfile.TemporaryDirectory() as tmp, patch.object(sys, "stdin", io.StringIO("".join(json.dumps(r) + "\n" for r in requests))), \
             patch.object(sys, "stdout", io.StringIO()) as out, patch.object(b.subprocess, "Popen", return_value=Process()) as proc:
            b.serve(Path(tmp))
            replies = [json.loads(line) for line in out.getvalue().splitlines()]
            self.assertTrue(all(t["inputSchema"]["additionalProperties"] is False for t in replies[0]["result"]["tools"]))
            self.assertIn("error", replies[1])
            self.assertEqual(proc.call_count, 1)
            self.assertEqual(proc.call_args.args[0][-1], "test")
            self.assertEqual(proc.call_args.kwargs["stdin"], subprocess.DEVNULL)
            self.assertIn("[E1 output truncated]", replies[2]["result"]["content"][0]["text"])
            self.assertTrue(replies[2]["result"]["isError"])
            self.assertTrue(c.jsonl(Path(tmp) / "broker.jsonl")[0]["truncated"])

    def test_monotonic_budget_and_dry_only_shortening(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            config = {"runs": 0, "started": time.time() + 9000, "started_monotonic": time.monotonic() - 2401,
                      "session_seconds": 999999, "dry_run": False}
            g.save(root / "session.json", config)
            self.assertEqual(h.command(root, "check"), 3)
            config.update(dry_run=True, session_seconds=1, started_monotonic=time.monotonic() - 2)
            g.save(root / "session.json", config)
            self.assertEqual(h.command(root, "test"), 3)
            self.assertFalse((root / "snapshot-01").exists())

    def test_store_scan_detects_content_even_with_different_name(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            tracked = root / "tracked"
            tracked.write_bytes(b"unique control")
            store = root / "store"
            store.mkdir()
            (store / "different-name").write_bytes(tracked.read_bytes())
            self.assertFalse(g.store_scan(store, [tracked])["passed"])
            (store / "different-name").write_bytes(b"other contents")
            self.assertTrue(g.store_scan(store, [tracked])["passed"])

    def test_store_scan_skips_unreadable_and_writer_visible_but_fails_held_out_bytes(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            trial = root / "repo/trials/e1-checked"
            (trial / "rust/src").mkdir(parents=True)
            (trial / "tasks.md").write_bytes(b"held-out task text")
            (trial / "rust/Cargo.toml").write_bytes(b"[package]\n")
            (trial / "rust/src/held.rs").write_bytes(b"held-out rust text")
            (trial / ".gitignore").write_bytes(b"/target\n")
            tracked = [trial / "tasks.md", trial / "rust/Cargo.toml", trial / "rust/src/held.rs", trial / ".gitignore"]
            store = root / "store"
            entry = store / ("a" * 32 + "-unrelated")
            entry.mkdir(parents=True)
            (entry / "Cargo.toml").write_bytes(b"[package]\n")
            (entry / ".gitignore").write_bytes(b"/target\n")
            locked = entry / "check.lock"
            locked.write_bytes(b"x" * len(b"held-out task text"))
            locked.chmod(0)
            # Unreadable content is skipped only when another uid owns it (EACCES). A jail runs as this
            # uid and can regain owner capabilities in a nested user namespace, so content this uid owns
            # but cannot read is fatal. Tests cannot chown, so a foreign owner is faked through getuid.
            foreign = patch.object(g.os, "getuid", return_value=os.getuid() + 1)
            try:
                with self.assertRaises(PermissionError):
                    g.store_scan(store, tracked, trial)
                with foreign:
                    scan = g.store_scan(store, tracked, trial)
                self.assertTrue(scan["passed"], scan["matches"])
                self.assertEqual(scan["unreadable_skipped"], [str(locked)])
                locked.chmod(0o644)
                (entry / "copy").write_bytes(b"held-out task text")
                scan = g.store_scan(store, tracked, trial)
                self.assertEqual(scan["matches"], [{"store": str(entry / "copy"), "tracked": [str(trial / "tasks.md")]}])
                (entry / "copy").unlink()
                # Only the files writers receive are exempt under rust/, not every file there.
                (entry / "held.rs").write_bytes(b"held-out rust text")
                self.assertEqual(g.store_scan(store, tracked, trial)["matches"],
                                 [{"store": str(entry / "held.rs"), "tracked": [str(trial / "rust/src/held.rs")]}])
                (entry / "held.rs").unlink()
                opaque = entry / "opaque"
                opaque.mkdir()
                (opaque / "known.txt").write_bytes(b"held-out task text")
                # A searchable but unlistable directory can still hold readable files: fatal for any owner.
                opaque.chmod(0o111)
                with self.assertRaises(PermissionError):
                    g.store_scan(store, tracked, trial)
                with foreign, self.assertRaises(PermissionError):
                    g.store_scan(store, tracked, trial)
                # An unsearchable directory is skipped only when another uid owns it (a root-owned 0700 Nix
                # build chroot on a real host); this uid's 0600 or 000 directory is fatal.
                for mode in [0o600, 0]:
                    opaque.chmod(mode)
                    with self.subTest(mode=oct(mode), owner="own"), self.assertRaises(PermissionError):
                        g.store_scan(store, tracked, trial)
                    with self.subTest(mode=oct(mode), owner="foreign"), foreign:
                        scan = g.store_scan(store, tracked, trial)
                        self.assertEqual((scan["passed"], scan["unreadable_skipped"]), (True, [str(opaque)]))
                # Any error at the search probe other than EACCES is fatal, even for a foreign owner.
                opaque.chmod(0)
                real_open = g.os.open
                def probe_fails(path, *args, **kwargs):
                    if str(path) == str(opaque) + "/.": raise OSError(5, "Input/output error", str(path))
                    return real_open(path, *args, **kwargs)
                with foreign, patch.object(g.os, "open", probe_fails), self.assertRaises(OSError) as raised:
                    g.store_scan(store, tracked, trial)
                self.assertEqual(raised.exception.errno, 5)
                opaque.chmod(0o755)
                # Any other metadata error at the first lookup stays fatal (Path.is_file() hid EIO).
                real = g.os.lstat
                def failing(path, *args, **kwargs):
                    if str(path) == str(opaque / "known.txt"): raise OSError(5, "Input/output error", str(path))
                    return real(path, *args, **kwargs)
                with patch.object(g.os, "lstat", failing), self.assertRaises(OSError) as raised:
                    g.store_scan(store, tracked, trial)
                self.assertEqual(raised.exception.errno, 5)
            finally:
                locked.chmod(0o644)
                if (entry / "opaque").exists(): (entry / "opaque").chmod(0o755)

    def test_store_scan_flags_trial_content_at_any_revision(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            trial = root / "repo/trials/e1-checked"
            (trial / "suites/common").mkdir(parents=True)
            tracked = trial / "suites/common/X1.rs"
            tracked.write_bytes(b"current revision")
            store = root / "store"
            older = store / ("c" * 32 + "-suites/common/X1.rs")
            older.parent.mkdir(parents=True)
            older.write_bytes(b"older revision")
            unrelated = store / ("d" * 32 + "-unrelated/common/X1.rs")
            unrelated.parent.mkdir(parents=True)
            unrelated.write_bytes(b"older revision")
            # Writer-visible manifests share names with many unrelated Rust trees.
            (trial / "rust").mkdir()
            (trial / "rust/Cargo.lock").write_bytes(b"pinned lock")
            (store / ("f" * 32 + "-rustc-src/rust")).mkdir(parents=True)
            (store / ("f" * 32 + "-rustc-src/rust/Cargo.lock")).write_bytes(b"other lock")
            self.assertEqual(g.store_scan(store, [tracked, trial / "rust/Cargo.lock"], trial)["matches"],
                             [{"store": str(store / ("c" * 32 + "-suites")), "trial_path": "suites/common/X1.rs"}])
            older.unlink()
            (store / ("e" * 32 + "-source/trials/e1-checked")).mkdir(parents=True)
            self.assertEqual(g.store_scan(store, [tracked], trial)["matches"],
                             [{"store": str(store / ("e" * 32 + "-source")), "trial_path": "source/trials/e1-checked"}])

    def test_gate_requires_current_both_arm_canaries_probe_and_store(self):
        with tempfile.TemporaryDirectory() as tmp, patch.object(g, "canary_key", return_value={"revision": 1}):
            root = Path(tmp)
            state = {"dry_run": False, "identity": {"cheap": {"cli": "pinned"}}}
            for arm in ["rust", "tide"]:
                g.save(root / (arm + "-canary.json"), {"passed": True, "key": {"revision": 1}, "arm": arm})
            g.save(root / "probe.json", {"passed": True, "key": {"revision": 1}, "writer": {"cli": "pinned"}})
            g.save(root / "store-before.json", {"passed": True, "key": {"revision": 1}})
            c.prerequisites(root, state)
            g.save(root / "tide-canary.json", {"passed": True, "key": {"revision": 0}, "arm": "tide"})
            with self.assertRaisesRegex(ValueError, "stale canary"):
                c.prerequisites(root, state)

    def test_final_grading_uses_saved_snapshot(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            session = root / "session"
            session.mkdir()
            (session / "final").mkdir()
            (session / "final/source.rs").write_text("stored control")
            g.save(session / "session.json", {"directory": str(root / "absent-writer"), "task": "A2"})
            with patch.object(h, "STATE", root), patch.object(sys, "argv", ["harness", "grade", str(session)]), \
                 patch.object(h, "grade", return_value={"structure": True, "results": []}) as grade:
                self.assertEqual(h.main(), 0)
                self.assertEqual(grade.call_args.args[2], "stored control")

    def test_jail_sees_only_the_store_closure_not_an_older_trial_copy(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            trial = root / "repo/trials/e1-checked"
            (trial / "suites/common").mkdir(parents=True)
            (trial / "suites/common/X1.rs").write_bytes(b"current revision")
            store = root / "store"
            closure = store / ("a" * 32 + "-loader")
            closure.mkdir(parents=True)
            (closure / "lib").write_text("loader control")
            older = store / ("b" * 32 + "-source/trials/e1-checked/suites/common/X1.rs")
            older.parent.mkdir(parents=True)
            older.write_bytes(b"older revision")
            work = root / "work"
            work.mkdir()
            # The host toolchain's real closure: the jail's /nix/store must list exactly it.
            real = h.store_closure([h.toolchain(h.RUST) / "bin"]) if h.STORE.is_dir() else []
            config = {"arm": "rust", "sandbox": True, "sysroot": "/toolchain", "vendor": "/vendor", "store_paths": [str(closure), *real]}
            listing = "import os; print(sorted(os.listdir('/nix/store')) if os.path.isdir('/nix/store') else [])"
            seen = h.run(["/usr/bin/python3", "-c", listing], config, work, timeout=30)
            self.assertEqual((seen["code"], seen["stdout"]), (0, f"{sorted(Path(p).name for p in real)}\n"), seen["stderr"])
            probe = "import os,sys; print([os.path.exists(p) for p in sys.argv[1:]])"
            result = h.run(["/usr/bin/python3", "-c", probe, str(closure / "lib"), str(older), str(store / older.relative_to(store).parts[0])],
                           config, work, timeout=30)
            self.assertEqual((result["code"], result["stdout"]), (0, "[True, False, False]\n"), result["stderr"])
            self.assertFalse(g.store_scan(store, [trial / "suites/common/X1.rs"], trial)["passed"])

    def test_jail_dies_with_a_killed_harness(self):
        mark = f"{41 + os.getpid() % 1000 / 1000:.3f}"
        child = ("import sys; sys.path.insert(0, sys.argv[1]); from pathlib import Path; import harness as h; "
                 "h.run(['/bin/sleep', sys.argv[3]], {'arm': 'rust', 'sandbox': True, 'sysroot': '/toolchain', "
                 "'vendor': '/vendor', 'store_paths': []}, Path(sys.argv[2]), timeout=100)")
        def sleepers():
            found = []
            for entry in Path("/proc").iterdir():
                try: argv = (entry / "cmdline").read_bytes().split(b"\0") if entry.name.isdigit() else []
                except OSError: continue
                if argv[1:2] == [mark.encode()] and argv[0].endswith(b"sleep"): found.append(int(entry.name))
            return found
        with tempfile.TemporaryDirectory() as tmp:
            # The harness runs in its own group, as the grader does under its guard.
            harness = subprocess.Popen([sys.executable, "-c", child, str(h.HERE), tmp, mark],
                                       start_new_session=True, stdin=subprocess.DEVNULL)
            try:
                deadline = time.monotonic() + 20
                while not sleepers() and time.monotonic() < deadline: time.sleep(0.1)
                self.assertTrue(sleepers(), "the jailed sleep never started")
            finally:
                os.killpg(harness.pid, signal.SIGKILL)
                harness.wait()
            deadline = time.monotonic() + 10
            while sleepers() and time.monotonic() < deadline: time.sleep(0.1)
            left = sleepers()
            for pid in left: os.kill(pid, signal.SIGKILL)
            self.assertEqual(left, [])

    def test_jail_memory_cap_kills_and_the_verdict_records_it(self):
        with tempfile.TemporaryDirectory() as tmp, patch.object(h, "MEMORY_MAX", 64 << 20):
            config = {"arm": "rust", "sandbox": True, "sysroot": "/toolchain", "vendor": "/vendor", "store_paths": []}
            control = h.run(["/usr/bin/python3", "-c", "b = bytearray(8 << 20); print('ok')"], config, Path(tmp), timeout=30)
            self.assertEqual((control["code"], control["stdout"]), (0, "ok\n"), control["stderr"])
            self.assertLess(control["memory_peak"], 64 << 20)
            self.assertGreater(control["cpu_usage_usec"], 0)
            self.assertNotIn("memory_limit_exceeded", control)
            # The allocating child dies; its parent survives and exits 0, yet the run fails.
            hog = "import subprocess,sys; subprocess.run([sys.executable, '-c', 'b = bytearray(256 << 20)'])"
            result = h.run(["/usr/bin/python3", "-c", hog], config, Path(tmp), timeout=30)
            self.assertEqual((result["code"], result["observed_exit"], result["memory_limit_exceeded"], result["infrastructure"]),
                             (137, 0, True, False))
            self.assertIn("memory limit exceeded", result["stderr"])

    def test_reporter_launches_nothing_without_the_recorded_limits(self):
        import jail_limits
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            own = Path(Path("/proc/self/cgroup").read_text().strip().split("::", 1)[1]).relative_to("/")
            limits = {"memory_max": str(h.MEMORY_MAX), "memory_swap_max": "0"}
            marker = root / "launched"
            for case in ["missing", "uncontrolled", "refused", "orphaned", "applied"]:
                fake = root / case
                scope = fake / own
                jail = scope / "jail"
                jail.mkdir(parents=True)
                if case != "missing":
                    (jail / "cgroup.controllers").write_text("cpu pids\n" if case == "uncontrolled" else "cpu memory pids\n")
                    (jail / "memory.max").write_text("max\n")
                    (jail / "memory.swap.max").write_text("max\n")
                    (jail / "memory.peak").write_text("4096\n")
                    (jail / "memory.events.local").write_text("low 0\nhigh 0\nmax 0\noom 0\noom_kill 0\noom_group_kill 0\n")
                    (jail / "cpu.stat").write_text("usage_usec 10\nuser_usec 5\nsystem_usec 5\n")
                state = root / (case + "-state")
                state.mkdir()
                (state / "env.json").write_text("{}")
                (state / "limits.json").write_text(json.dumps(limits))
                parent = os.getpid() + (1 << 22) if case == "orphaned" else os.getppid()
                argv = ["jail_limits.py", str(parent), str(state), "--", sys.executable, "-c", f"open({str(marker)!r}, 'w').close()"]
                real_set = jail_limits.set_value
                # "refused": the kernel keeps memory.max at "max" whatever the reporter writes.
                def set_value(path, value):
                    if not (case == "refused" and path.name == "memory.max"): real_set(path, value)
                with self.subTest(case=case), patch.object(jail_limits, "CGROUP_ROOT", fake), \
                     patch.object(sys, "argv", argv), patch.object(jail_limits, "set_value", set_value), \
                     patch.object(jail_limits.ctypes, "CDLL", lambda *_a, **_k: type("C", (), {"prctl": staticmethod(lambda *_a: 0)})):
                    code = jail_limits.main()
                    written = json.loads((state / "report.json").read_text())
                    if case == "applied":
                        self.assertEqual((code, marker.exists(), written["oom_kill"], written["cpu_usage_usec"]), (0, True, 0, 10))
                        # The reporter left the capped leaf; the cap and the jail's process sit on jail/.
                        self.assertEqual([(scope / "reporter/cgroup.procs").read_text(), (scope / "cgroup.subtree_control").read_text(),
                                          (jail / "memory.max").read_text(), (jail / "memory.swap.max").read_text(),
                                          (jail / "cgroup.procs").read_text()],
                                         ["0\n", "+memory\n", f"{h.MEMORY_MAX}\n", "0\n", "0\n"])
                        marker.unlink()
                    else:
                        self.assertEqual((code, marker.exists()), (jail_limits.SETUP_EXIT, False))
                        self.assertIn("gone" if case == "orphaned" else "", written["setup_error"])

    def test_a_lowered_score_swarm_cannot_turn_its_cap_failure_into_infrastructure(self):
        # The jail lowers its own OOM score, then forks past the cap. The OOM killer must pick only
        # jail processes, never the reporter, so every run is a candidate memory failure.
        swarm = ("import os,time\nopen('/proc/self/oom_score_adj','w').write('0')\nkids=[]\n"
                 "for i in range(40):\n pid=os.fork()\n if pid==0:\n  b=bytearray(3<<20)\n  time.sleep(2)\n  os._exit(0)\n kids.append(pid)\n"
                 "for k in kids: os.waitpid(k,0)\nprint('done')\n")
        config = {"arm": "rust", "sandbox": True, "sysroot": "/toolchain", "vendor": "/vendor", "store_paths": []}
        with patch.object(h, "MEMORY_MAX", 96 << 20):
            for attempt in range(5):
                with self.subTest(attempt=attempt), tempfile.TemporaryDirectory() as tmp:
                    result = h.run(["/usr/bin/python3", "-c", swarm], config, Path(tmp), timeout=60)
                    self.assertEqual((result["code"], result["infrastructure"], result.get("memory_limit_exceeded")),
                                     (137, False, True), result["stderr"][-300:])

    def test_only_local_limit_exhaustion_is_a_candidate_memory_failure(self):
        config = {"arm": "rust", "sandbox": True, "sysroot": "/toolchain", "vendor": "/vendor", "store_paths": []}
        for name, events, expected in [("local", {"max": 7, "oom": 1, "oom_kill": 1}, (137, False, True)),
                                       ("external", {"max": 0, "oom": 0, "oom_kill": 1}, (2, True, False)),
                                       ("ambiguous", {"max": 3, "oom": 0, "oom_kill": 1}, (2, True, False))]:
            class Process:
                returncode = 0
                pid = 1
                def __init__(self, argv, **kwargs):
                    limit_dir = Path(argv[argv.index(str(h.HERE / "jail_limits.py")) + 2])
                    (limit_dir / "report.json").write_text(json.dumps({"memory_peak": 64 << 20, "cpu_usage_usec": 1,
                        "memory_max": str(h.MEMORY_MAX), "memory_swap_max": "0", **events}))
                    h.os.write(kwargs["pass_fds"][0], b'{"exit-code": 0}\n')
                def communicate(self, *_args, **_kwargs):
                    return b"", b""
            with self.subTest(name=name), tempfile.TemporaryDirectory() as tmp, patch.object(h.subprocess, "Popen", Process):
                result = h.run(["/binary"], config, Path(tmp))
                self.assertEqual((result["code"], result["infrastructure"], result.get("memory_limit_exceeded", False)), expected)
                self.assertEqual(result["memory_events"], events)

    def test_identity_records_pinned_jobs_limits_and_store_closure(self):
        configs = {arm: {"sysroot": "/toolchain", "driver": "/driver", "std": "/std", "store_paths": ["/nix/store/x-" + arm]}
                   for arm in ["rust", "tide"]}
        with patch.object(g, "output", return_value="control"), patch.object(g, "digest", return_value="0" * 64), \
             patch.object(g, "clean"), patch.object(g, "canary_key", return_value={}), patch.dict(h.os.environ, {"CARGO_BUILD_JOBS": "6"}):
            record = g.identity(configs, "/pinned/claude", g.MODEL, 1)
        self.assertEqual(record["jail_limits"]["cargo_build_jobs"], "4")
        self.assertEqual((record["jail_limits"]["memory_max"], record["jail_limits"]["memory_swap_max"],
                          record["jail_limits"]["tmpfs_size"]), (h.MEMORY_MAX, 0, h.TMPFS_SIZE))
        self.assertEqual(record["store_closure"], {"rust": ["/nix/store/x-rust"], "tide": ["/nix/store/x-tide"]})

    def test_runtime_tmpdir_is_tmpfs_in_both_arms(self):
        for arm in ["rust", "tide"]:
            config = {"arm": arm, "sandbox": True, "sysroot": "/toolchain", "vendor": "/vendor",
                      "driver": "/driver", "std": "/std", "deps": []}
            cmd = h.sandbox_command(["/binary"], config, Path("/work"), read_only=["/binary"])
            at = cmd.index("--tmpfs")
            self.assertEqual(cmd[at - 2:at + 2], ["--size", str(h.TMPFS_SIZE), "--tmpfs", "/tmp"])
            self.assertEqual(h.environment(config, Path("/work"))["TMPDIR"], "/tmp")

    def test_stream_monitor_checks_init_exit_audit_and_clock_without_a_model(self):
        init = {"type": "system", "subtype": "init", "tools": sorted(g.TOOLS), "plugins": [],
                "mcp_servers": [{"name": "e1", "status": "connected"}], "model": g.MODEL}
        cases = [(init, [], 0, None), (dict(init, tools=["Bash"]), [], 0, "init_failure"),
                 (init, [], 7, "cli_error"),
                 (init, [{"type": "assistant", "message": {"model": g.MODEL, "content":
                     [{"type": "tool_use", "name": "Bash", "input": {}}]}}], 0, "log_audit")]
        for first, middle, code, fault in cases:
            with self.subTest(fault=fault), tempfile.TemporaryDirectory() as tmp:
                root = Path(tmp)
                directory = root / "writer"
                directory.mkdir()
                (directory / "prompt.md").write_text("synthetic control")
                g.save(root / "session.json", {"directory": str(directory), "started": None, "runs": 0})
                (root / "broker.jsonl").write_text(' {"event":"start"}\n{"event":"stop"}\n')
                events = [first, *middle, {"type": "result", "subtype": "success"}]
                script = "import json,sys; sys.stdin.read(); " + "; ".join(
                    "print(" + repr(json.dumps(event)) + ", flush=True)" for event in events) + f"; sys.exit({code})"
                with patch.object(c, "writer_argv", return_value=[sys.executable, "-c", script]):
                    result = c.run_writer(root, "unused", g.MODEL, seconds=5)
                self.assertEqual(result["fault"], fault, result)
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            directory = root / "writer"
            directory.mkdir()
            (directory / "prompt.md").write_text("synthetic control")
            g.save(root / "session.json", {"directory": str(directory), "started": None, "runs": 0, "dry_run": True})
            script = "import time; print(" + repr(json.dumps(init)) + ", flush=True); time.sleep(10)"
            with patch.object(c, "writer_argv", return_value=[sys.executable, "-c", script]):
                result = c.run_writer(root, "unused", g.MODEL, seconds=0.2)
            self.assertIsNone(result["fault"])
            self.assertEqual(result["end_reason"], "time_limit")
            self.assertLess(result["wall_seconds"], 1)


class FixRound1Tests(unittest.TestCase):
    def test_probe_accepts_complete_requests_with_actual_permission_frames(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            state = {"dry_run": False, "cli": "unused", "model": g.MODEL, "identity": {"cheap": {}}}
            def writer(session, *_args, **kwargs):
                inside = root / "probe-writer/inside.txt"
                inside.write_text("control\n")
                events = []
                calls = [("Write", {"file_path": str(inside), "content": "control\n"}, False),
                         ("Read", {"file_path": str(inside), "offset": 1, "limit": 1}, False)]
                calls += [("Read", {"file_path": str(path), "offset": 1, "limit": 1}, True)
                          for path in [h.ROOT / "AGENTS.md", Path("/etc/hostname")]]
                calls.append(("Write", {"file_path": str(root / "probe-outside"), "content": c.PROBE_CONTENT}, True))
                for index, (name, args, denied) in enumerate(calls):
                    message = "Denied by restricted mode" if denied else "control"
                    events.extend([{"type": "tool_use", "id": str(index), "name": name, "input": args},
                                   {"type": "tool_result", "tool_use_id": str(index), "is_error": denied, "content": message}])
                    if denied:
                        events.append({"type": "system", "subtype": "permission_denied", "tool_name": name,
                                       "tool_use_id": str(index), "decision_reason_type": "other",
                                       "decision_reason": c.DENIAL_REASON, "message": message})
                (session / "writer.jsonl").write_text("".join(json.dumps(event) + "\n" for event in events))
                return {"fault": None}
            with patch.object(c, "run_writer", side_effect=writer), patch.object(g, "canary_key", return_value={}):
                c.probe(root, state)
            self.assertTrue(json.loads((root / "probe.json").read_text())["passed"])

    def test_batch_cancels_groups_before_waiting_for_workers(self):
        events = []
        class Pool:
            def __init__(self, **kwargs): pass
            def map(self, *_args): raise KeyboardInterrupt()
            def shutdown(self, **kwargs): events.append("shutdown")
        with tempfile.TemporaryDirectory() as tmp, patch.object(c, "ThreadPoolExecutor", Pool), \
             patch.object(c, "stop_active", side_effect=lambda: events.append("stop")):
            state = {"dry_run": True, "order": [], "jobs": 1}
            with self.assertRaises(KeyboardInterrupt): c.batch(Path(tmp), state, 1)
        self.assertEqual(events, ["stop", "shutdown"])

    def test_recovery_accepts_reused_and_ungranted_identity_without_signals(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            boot = Path("/proc/sys/kernel/random/boot_id").read_text().strip()
            saved = {"pid": 123, "group": 123, "start": "old", "boot": boot}
            g.save(root / "process.json", saved)
            with patch.object(c, "process_identity", return_value=dict(saved, start="different")), \
                 patch.object(c.os, "killpg") as killed:
                c.reclaim_writer(root)
                killed.assert_not_called()
            (root / "process.json").unlink()
            g.save(root / "launch.json", {"argv": ["unidentified previous writer"]})
            c.reclaim_writer(root)

    def test_probe_requires_valid_requests_and_structured_confinement_denial(self):
        use = {"type": "tool_use", "id": "r", "name": "Read",
               "input": {"file_path": "/etc/hostname", "offset": 1, "limit": 1}}
        denial = {"type": "system", "subtype": "permission_denied", "tool_name": "Read", "tool_use_id": "r",
                  "decision_reason_type": "other", "decision_reason": c.DENIAL_REASON,
                  "message": "Read refused because the path is outside the working directory"}
        result = {"is_error": True, "content": denial["message"]}
        self.assertTrue(c.probe_denied(use, result, [denial]))
        self.assertFalse(c.probe_denied(use, result, []))
        self.assertFalse(c.probe_denied(use, {"is_error": True, "content": "InputValidationError: offset"}, [denial]))
        bad = dict(use, input={"file_path": "/etc/hostname", "offset": -1, "limit": 1})
        self.assertFalse(c.probe_denied(bad, result, [denial]))
        self.assertFalse(c.probe_denied(dict(use, name="Write", input={"file_path": "/tmp/outside"}), result, [denial]))
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            state = {"dry_run": False, "cli": "unused", "model": g.MODEL, "identity": {"cheap": {}}}
            def writer(session, *_args, **kwargs):
                directory = root / "probe-writer"
                events = []
                for index, (name, args, error, text) in enumerate([
                    ("Write", {"file_path": str(directory / "inside.txt"), "content": "control"}, False, "ok"),
                    ("Read", {"file_path": str(directory / "inside.txt")}, False, "control"),
                    ("Read", {"file_path": str(h.ROOT / "AGENTS.md"), "offset": -1}, True, "InputValidationError"),
                    ("Read", {"file_path": "/etc/hostname", "offset": -1}, True, "InputValidationError"),
                    ("Write", {"file_path": str(root / "probe-outside")}, True, "InputValidationError"),
                ]):
                    events.extend([{"type": "tool_use", "id": str(index), "name": name, "input": args},
                                   {"type": "tool_result", "tool_use_id": str(index), "is_error": error, "content": text}])
                (session / "writer.jsonl").write_text("".join(json.dumps(event) + "\n" for event in events))
                return {"fault": None}
            with patch.object(c, "run_writer", side_effect=writer), patch.object(g, "canary_key", return_value={}):
                c.probe(root, state)
            self.assertFalse(json.loads((root / "probe.json").read_text())["passed"])

    def test_last_command_captures_final_and_monitor_ends_writer(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            writer = root / "writer"
            (writer / "src").mkdir(parents=True)
            entry = writer / "src/lib.rs"
            entry.write_text("before exhaustion")
            (writer / "prompt.md").write_text("synthetic control")
            g.save(root / "session.json", {"directory": str(writer), "task": "A2", "arm": "rust", "sandbox": False,
                                           "vendor": "/vendor", "sysroot": "/toolchain", "runs": 19, "started": None})
            init = {"type": "system", "subtype": "init", "tools": sorted(g.TOOLS), "plugins": [],
                    "mcp_servers": [{"name": "e1", "status": "connected"}], "model": g.MODEL}
            fixture = root / "writer-control.py"
            fixture.write_text('''import json, sys, time
from pathlib import Path
sys.path.insert(0, sys.argv[1])
import harness as h
print(sys.argv[3], flush=True)
h.checks = lambda *args: [{"code": 0, "stdout": "", "stderr": ""}]
h.command(Path(sys.argv[2]), "check")
Path(sys.argv[4]).write_text("late edit")
time.sleep(0.1)
Path(sys.argv[2], "broker.jsonl").write_text('{"mode":"check","exit":0,"completed_run":20}\\n')
time.sleep(30)
''')
            argv = [sys.executable, str(fixture), str(h.HERE), str(root), json.dumps(init), str(entry)]
            with patch.object(c, "writer_argv", return_value=argv):
                result = c.run_writer(root, "unused", g.MODEL, seconds=5)
            self.assertIsNone(result["fault"], result)
            self.assertEqual(result["end_reason"], "run_limit")
            self.assertEqual((root / "final/source.rs").read_text(), "before exhaustion")
            self.assertEqual(json.loads((root / "snapshot-20/results.json").read_text())[0]["code"], 0)
            self.assertNotIn("deadline", json.loads((root / "session.json").read_text()))
            self.assertEqual(c.jsonl(root / "broker.jsonl")[-1]["completed_run"], 20)
            self.assertFalse(c.group_alive(json.loads((root / "process.json").read_text())["group"]))

    def test_grading_setup_exit_and_timeout_never_resample_the_writer(self):
        for failure in [{"exit": 2}, {"exit": 1, "infrastructure": True}]:
            records = []
            def runner(attempt):
                return {"fault": None, "end_reason": "done", "grades": {"first": failure if attempt == 1 else {"exit": 0}, "final": {"exit": 0}}}
            with self.assertRaisesRegex(RuntimeError, "never the writer"):
                c.run_slot(0, ["A2", "rust", 1], runner, records.append)
            self.assertEqual(records, [])
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            writer = root / "writer"
            (writer / "src").mkdir(parents=True)
            (writer / "src/lib.rs").write_text("generic control")
            g.save(root / "session.json", {"directory": str(writer), "task": "A2"})
            class Process:
                pid = 123456
                waits = 0
                def __init__(self, argv, **kwargs):
                    name = "first" if "--first" in argv else "final"
                    g.save(root / (name + "-grade.json"), {"structure": True, "results": []})
                def wait(self, timeout=None):
                    self.waits += 1
                    if self.waits == 1: raise subprocess.TimeoutExpired("grader", timeout)
                    return -9
            readers = []
            def start(argv, *_args, **_kwargs):
                read, grant = os.pipe()
                readers.append(read)
                return Process(argv), object(), grant
            try:
                with patch.object(c, "start_guard", side_effect=start), patch.object(c, "end_guard"):
                    grades = c.freeze_and_grade(root)
                self.assertTrue(all(grade["infrastructure"] and grade["exit"] == 2 for grade in grades.values()))
            finally:
                for fd in readers: os.close(fd)
                for path in [writer, writer / "src"]: path.chmod(0o755)

    def test_grade_preserves_jail_setup_failure_as_infrastructure(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            (root / "final").mkdir()
            (root / "final/source.rs").write_text("generic control")
            g.save(root / "session.json", {"task": "A2", "directory": str(root / "absent")})
            with patch.object(sys, "argv", ["harness", "grade", str(root)]), patch.object(h, "STATE", root), \
                 patch.object(h, "grade", return_value={"structure": True, "results": [{"code": 2, "infrastructure": True}]}):
                self.assertEqual(h.main(), 2)
            self.assertTrue(json.loads((root / "final-grade.json").read_text())["infrastructure"])

    def test_interrupted_parent_cannot_leave_a_live_writer_on_resume(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            writer = root / "writer"
            writer.mkdir()
            (writer / "prompt.md").write_text("synthetic control")
            g.save(root / "session.json", {"directory": str(writer), "started": None, "runs": 0})
            sleeper = root / "sleep-control.py"
            sleeper.write_text('import time\nfrom pathlib import Path\nPath(__file__).with_name("writer-started").touch()\ntime.sleep(30)\n')
            fixture = root / "coordinator-control.py"
            fixture.write_text('''import sys
from pathlib import Path
sys.path.insert(0, sys.argv[1])
import coordinator as c
c.writer_argv = lambda *args: [sys.executable, sys.argv[3]]
c.run_writer(Path(sys.argv[2]), "unused", c.g.MODEL, seconds=30)
''')
            parent = subprocess.Popen([sys.executable, str(fixture), str(h.HERE), str(root), str(sleeper)],
                                      stdin=subprocess.DEVNULL, stdout=subprocess.DEVNULL,
                                      stderr=subprocess.PIPE, start_new_session=True)
            try:
                until = time.monotonic() + 5
                while not (root / "process.json").exists() or not (root / "writer-started").exists():
                    if parent.poll() is not None: self.fail(parent.stderr.read().decode())
                    if time.monotonic() >= until: self.fail("guard identity not published")
                    time.sleep(0.02)
                group = json.loads((root / "process.json").read_text())["group"]
                os.kill(parent.pid, signal.SIGKILL)
                parent.wait(timeout=5)
                c.reclaim_writer(root)
                self.assertFalse(c.group_alive(group))
            finally:
                if parent.poll() is None: parent.kill(); parent.wait(timeout=5)
                parent.stderr.close()
                c.reclaim_writer(root)

    def test_tmpfs_precedes_executable_bind_for_external_tmp_run(self):
        with tempfile.TemporaryDirectory(dir="/tmp") as tmp:
            root = Path(tmp)
            binary = root / "output/test"
            binary.parent.mkdir()
            binary.write_text("control")
            work = root / "scratch"
            work.mkdir()
            config = {"arm": "rust", "sysroot": "/toolchain", "vendor": "/vendor"}
            cmd = h.sandbox_command([str(binary)], config, work, read_only=[binary])
            self.assertLess(cmd.index("--tmpfs"), cmd.index(str(binary)))
            self.assertIn([str(work), str(work)], [cmd[i+1:i+3] for i, arg in enumerate(cmd) if arg == "--bind"])


class FixRound2Tests(unittest.TestCase):
    def test_guarded_grading_preserves_the_child_exit_without_compiling(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            writer = root / "writer"
            (writer / "src").mkdir(parents=True)
            (writer / "src/lib.rs").write_text("generic control")
            g.save(root / "session.json", {"directory": str(writer), "task": "A2"})
            fixture = root / "grade-control.py"
            fixture.write_text('''import json, sys
from pathlib import Path
Path(sys.argv[1], sys.argv[2] + "-grade.json").write_text(json.dumps({"structure":False,"results":[{"code":1}]}))
sys.exit(1)
''')
            real_start = c.start_guard
            def start(argv, directory, **kwargs):
                name = "first" if "--first" in argv else "final"
                return real_start([sys.executable, str(fixture), str(root), name], directory, **kwargs)
            try:
                with patch.object(c, "start_guard", side_effect=start):
                    grades = c.freeze_and_grade(root)
                self.assertEqual({name: row["exit"] for name, row in grades.items()}, {"first": 1, "final": 1})
                self.assertTrue(all(not row["infrastructure"] for row in grades.values()))
            finally:
                for path in [writer, writer / "src"]: path.chmod(0o755)

    def test_pidfd_failure_has_no_numeric_signal_fallback(self):
        class Handle:
            closed = False
            def fileno(self): return 91
        with patch.object(c.signal, "pidfd_send_signal", side_effect=PermissionError("blocked")), \
             patch.object(c.os, "killpg") as group_signal, patch.object(c.os, "kill") as pid_signal:
            with self.assertRaisesRegex(RuntimeError, "refusing further attempts"): c.signal_guard(Handle())
            group_signal.assert_not_called()
            pid_signal.assert_not_called()

    def test_recovery_identity_reused_after_verification_never_signals(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            boot = Path("/proc/sys/kernel/random/boot_id").read_text().strip()
            saved = {"pid": 123, "group": 123, "start": "original", "boot": boot}
            g.save(root / "process.json", saved)
            observed = [dict(saved), dict(saved, start="reused")]
            with patch.object(c, "process_identity", side_effect=observed), \
                 patch.object(c, "group_alive", return_value=True), patch.object(c.time, "sleep"), \
                 patch.object(c.os, "killpg") as group_signal, patch.object(c.os, "kill") as pid_signal, \
                 patch.object(c.signal, "pidfd_send_signal") as handle_signal:
                c.reclaim_writer(root)
                group_signal.assert_not_called()
                pid_signal.assert_not_called()
                handle_signal.assert_not_called()

    def test_recovery_waits_without_signals_and_refuses_if_group_stays_live(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            boot = Path("/proc/sys/kernel/random/boot_id").read_text().strip()
            saved = {"pid": 123, "group": 123, "start": "original", "boot": boot}
            g.save(root / "process.json", saved)
            with patch.object(c, "process_identity", return_value=saved), \
                 patch.object(c, "group_alive", side_effect=[True, False]), patch.object(c.time, "sleep") as slept, \
                 patch.object(c.signal, "pidfd_send_signal") as signalled:
                c.reclaim_writer(root)
                slept.assert_called_once()
                signalled.assert_not_called()
            with patch.object(c, "process_identity", return_value=saved), \
                 patch.object(c, "group_alive", return_value=True), \
                 patch.object(c.time, "monotonic", side_effect=[0, 11]), \
                 patch.object(c.signal, "pidfd_send_signal") as signalled:
                with self.assertRaisesRegex(ValueError, "still alive"): c.reclaim_writer(root)
                signalled.assert_not_called()

    def test_cancellation_racing_reaping_keeps_handle_until_signal_finishes(self):
        entered, reaped = threading.Event(), threading.Event()
        actions = []
        class Handle:
            closed = False
            def fileno(self): return 91
            def close(self):
                self.closed = True
                actions.append("closed")
        class Process:
            pid = 123
        handle, proc = Handle(), Process()
        def reap():
            if not entered.wait(2): return
            actions.append("pid reused")
            reaped.set()
            c.release_guard(proc, handle)
        def send(fd, sig):
            entered.set()
            self.assertTrue(reaped.wait(2))
            self.assertEqual(fd, 91)
            self.assertEqual(sig, signal.SIGTERM)
            self.assertFalse(handle.closed)
            actions.append("original guard signalled through pidfd")
        with patch.dict(c.ACTIVE, {proc.pid: handle}, clear=True), \
             patch.object(c.signal, "pidfd_send_signal", side_effect=send), \
             patch.object(c.os, "killpg") as group_signal, patch.object(c.os, "kill") as pid_signal:
            worker = threading.Thread(target=reap)
            worker.start()
            try: c.stop_active()
            finally: worker.join(timeout=2)
            self.assertFalse(worker.is_alive())
            self.assertTrue(handle.closed)
            self.assertEqual(actions, ["pid reused", "original guard signalled through pidfd", "closed"])
            group_signal.assert_not_called()
            pid_signal.assert_not_called()
            # Even a snapshotted handle cannot be signalled after close/fd reuse.
            c.signal_guard(handle)
            self.assertEqual(actions.count("original guard signalled through pidfd"), 1)


class FixRound3Tests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name)

    def tearDown(self):
        for path in self.root.rglob("*"):
            if path.is_dir() and not path.is_symlink(): path.chmod(0o755)
        self.tmp.cleanup()

    def state(self, jobs=1, real=False):
        return {"dry_run": not real, "jobs": jobs, "order": [["A2", "rust", 1], ["A3", "tide", 1]][:jobs],
                "configs": {"rust": {}, "tide": {}}, "cli": "unused", "model": g.MODEL, "seconds": None,
                "identity": {"cheap": {"head": "fixed"}}}

    def prepare(self, task, arm, directory, config, *, session):
        (directory / "src").mkdir(parents=True)
        session.mkdir(parents=True)
        (directory / "src/lib.rs").write_text("generic control")
        (directory / "prompt.md").write_text("synthetic control")
        g.save(session / "session.json", {"task": task, "arm": arm, "directory": str(directory), "runs": 0})

    def grant(self, session, callback):
        g.save(session / "launch.json", {})
        g.save(session / "process.json", {"boot": "previous boot"})
        callback()

    def green(self):
        return {"first": {"exit": 0}, "final": {"exit": 0}}

    def test_one_interruption_resumes_as_one_fault_and_one_fresh_writer(self):
        calls = []
        def stopped(session, *_args, on_started=None, cancelled=None, **kwargs):
            self.grant(session, on_started)
            calls.append(session.name)
            cancelled.set()
            return {"fault": "host_interruption", "end_reason": "host_interruption"}
        def healthy(session, *_args, on_started=None, **kwargs):
            self.grant(session, on_started)
            calls.append(session.name)
            return {"fault": None, "end_reason": "done"}
        with patch.object(h, "prepare", side_effect=self.prepare), patch.object(c, "run_writer", side_effect=stopped):
            with self.assertRaises(c.RunCancelled): c.batch(self.root, self.state(), 1)
        row = json.loads((self.root / "slot-000.json").read_text())
        self.assertEqual(len(row["attempts"]), 1)
        self.assertFalse((self.root / "sessions/000-2").exists())
        with patch.object(h, "prepare", side_effect=self.prepare), patch.object(c, "run_writer", side_effect=healthy), \
             patch.object(c, "freeze_and_grade", return_value=self.green()):
            c.batch(self.root, self.state(), 1)
        row = json.loads((self.root / "slot-000.json").read_text())
        self.assertEqual(calls, ["000-1", "000-2"])
        self.assertEqual(sum(bool(a.get("fault")) for a in row["attempts"]), 1)
        self.assertEqual(row["reruns"], 1)
        self.assertTrue(row["complete"])
        self.assertFalse(row["failed_infrastructure"])

    def test_unstarted_legacy_placeholders_are_not_faults_on_resume(self):
        for actual_first in [False, True]:
            root = self.root / str(actual_first)
            root.mkdir()
            first = root / "sessions/000-1"
            first.mkdir(parents=True)
            g.save(first / "launch.json", {})
            if actual_first: g.save(first / "process.json", {"boot": "old boot"})
            second = root / "sessions/000-2"
            second.mkdir()
            g.save(second / "launch.json", {})
            g.save(root / "slot-000.json", {"complete": False, "attempts": [
                {"fault": "host_interruption", "end_reason": "host_interruption"},
                {"fault": "host_interruption", "end_reason": "host_interruption"}]})
            calls = []
            def healthy(session, *_args, on_started=None, **kwargs):
                self.grant(session, on_started)
                calls.append(session.name)
                return {"fault": None, "end_reason": "done"}
            with patch.object(h, "prepare", side_effect=self.prepare), patch.object(c, "run_writer", side_effect=healthy), \
                 patch.object(c, "freeze_and_grade", return_value=self.green()):
                c.batch(root, self.state(), 1)
            row = json.loads((root / "slot-000.json").read_text())
            self.assertEqual(calls, ["000-2" if actual_first else "000-1"])
            self.assertEqual(row["reruns"], int(actual_first))
            self.assertFalse(row["failed_infrastructure"])

    def test_jobs_two_stop_and_identity_change_never_create_phantom_faults(self):
        for stop in ["cancel", "identity"]:
            root = self.root / stop
            root.mkdir()
            local, first = threading.local(), threading.Event()
            calls = []
            def prepare(task, arm, directory, config, *, session):
                self.prepare(task, arm, directory, config, session=session)
                local.slot = session.name
                if session.name.startswith("001"): self.assertTrue(first.wait(2))
            def cheap(*_args):
                return {"head": "changed" if getattr(local, "slot", "").startswith("001") else "fixed"}
            def stopped(session, *_args, on_started=None, cancelled=None, **kwargs):
                self.grant(session, on_started)
                calls.append(session.name)
                first.set()
                if stop == "cancel": cancelled.set()
                else: self.assertTrue(cancelled.wait(2))
                return {"fault": "host_interruption", "end_reason": "host_interruption"}
            state = self.state(2, real=stop == "identity")
            with patch.object(c, "prerequisites"), patch.object(h, "prepare", side_effect=prepare), \
                 patch.object(g, "cheap", side_effect=cheap), patch.object(c, "run_writer", side_effect=stopped):
                with self.assertRaises(RuntimeError): c.batch(root, state, 2)
            self.assertEqual(calls, ["000-1"])
            self.assertFalse((root / "slot-001.json").exists())
            self.assertEqual(len(json.loads((root / "slot-000.json").read_text())["attempts"]), 1)
            def healthy(session, *_args, on_started=None, **kwargs):
                self.grant(session, on_started)
                calls.append(session.name)
                return {"fault": None, "end_reason": "done"}
            with patch.object(c, "prerequisites"), patch.object(h, "prepare", side_effect=self.prepare), \
                 patch.object(g, "cheap", return_value={"head": "fixed"}), \
                 patch.object(g, "store_scan", return_value={"passed": True}), patch.object(g, "canary_key", return_value={}), \
                 patch.object(c, "run_writer", side_effect=healthy), patch.object(c, "freeze_and_grade", return_value=self.green()):
                c.batch(root, state, 2)
            for i, reruns in [(0, 1), (1, 0)]:
                row = json.loads((root / f"slot-{i:03}.json").read_text())
                self.assertEqual(row["reruns"], reruns)
                self.assertTrue(row["complete"])
                self.assertFalse(row["failed_infrastructure"])

    def test_grading_fault_retries_snapshot_and_resume_never_relaunches_writer(self):
        writers, grades = [], []
        def writer(session, *_args, on_started=None, **kwargs):
            self.grant(session, on_started)
            writers.append(session.name)
            return {"fault": None, "end_reason": "done"}
        def grade(session, cancelled=None):
            record = json.loads((session / "writer-result.json").read_text())
            self.assertEqual(record["end_reason"], "done")
            self.assertEqual(record["final_sha256"], g.digest(session / "final/source.rs"))
            grades.append(session.name)
            return {"final": {"exit": 2, "infrastructure": True}} if len(grades) == 1 else self.green()
        with patch.object(h, "prepare", side_effect=self.prepare), patch.object(c, "run_writer", side_effect=writer), \
             patch.object(c, "freeze_and_grade", side_effect=grade):
            c.batch(self.root, self.state(), 1)
        self.assertEqual(writers, ["000-1"])
        self.assertEqual(grades, ["000-1", "000-1"])
        row = json.loads((self.root / "slot-000.json").read_text())
        self.assertEqual(row["reruns"], 0)
        self.assertTrue(row["final_pass"])
        # Model a stop after writer capture but before grading was committed to the slot.
        g.save(self.root / "slot-000.json", {"complete": False, "attempts": [
            {"fault": "host_interruption", "end_reason": "host_interruption"}]})
        with patch.object(c, "run_writer", side_effect=AssertionError("frozen writer cannot relaunch")), \
             patch.object(c, "freeze_and_grade", return_value=self.green()):
            c.batch(self.root, self.state(), 1)
        self.assertEqual(writers, ["000-1"])
        self.assertEqual(json.loads((self.root / "slot-000.json").read_text())["reruns"], 0)

    def test_frozen_resume_repeats_the_full_writer_classification(self):
        for case, expected in [("budget", "harness_setup"), ("exit2", "harness_setup"),
                               ("init", "init_failure"), ("lifecycle", "broker_crash"),
                               ("audit", "log_audit"), ("healthy", None)]:
            with self.subTest(case=case):
                root = self.root / case
                session = root / "sessions/000-1"
                directory = root / "w/000-1"
                self.prepare("A2", "rust", directory, {}, session=session)
                self.grant(session, lambda: None)
                c.freeze_final(session)
                init = {"type": "system", "subtype": "init", "tools": sorted(g.TOOLS), "plugins": [],
                        "mcp_servers": [{"name": "e1", "status": "connected"}], "model": g.MODEL}
                if case == "init": init["tools"] = ["Bash"]
                events = [init, {"type": "result", "subtype": "success", "is_error": False}]
                if case == "audit": events.append({"type": "tool_use", "name": "Bash", "input": {}})
                (session / "writer.jsonl").write_text("".join(json.dumps(e) + "\n" for e in events))
                broker = [{"event": "start"}, {"event": "stop"}]
                if case == "lifecycle": broker.pop()
                if case == "exit2": broker.append({"exit": 2})
                (session / "broker.jsonl").write_text("".join(json.dumps(e) + "\n" for e in broker))
                g.save(session / "process-exit.json", {"exit": 0})
                if case == "budget":
                    g.save(session / "budget.json", {"reason": "run_limit", "completed_run": 20,
                                                   "infrastructure": True})
                g.save(root / "slot-000.json", {"complete": False, "attempts": [
                    {"fault": "host_interruption", "end_reason": "host_interruption"}]})
                with patch.object(h, "prepare", side_effect=self.prepare), \
                     patch.object(c, "run_writer", return_value={"fault": "cli_error", "end_reason": "cli_error"}) as writer, \
                     patch.object(c, "freeze_and_grade", return_value=self.green()) as grade:
                    c.batch(root, self.state(), 1)
                row = json.loads((root / "slot-000.json").read_text())
                self.assertEqual(row["attempts"][0]["fault"], expected)
                self.assertEqual(writer.call_count, int(expected is not None))
                self.assertEqual(grade.call_count, int(expected is None))
                if expected is None: self.assertEqual(row["attempts"][0]["end_reason"], "done")

    def test_frozen_resume_refuses_changed_final_hash_before_grading_or_overwriting(self):
        session = self.root / "sessions/000-1"
        self.prepare("A2", "rust", self.root / "w/000-1", {}, session=session)
        original_hash = c.freeze_final(session)
        saved = {"fault": None, "end_reason": "done", "session": str(session), "final_sha256": original_hash}
        g.save(session / "writer-result.json", saved)
        original_record = (session / "writer-result.json").read_bytes()
        final = session / "final/source.rs"
        final.chmod(0o644)
        final.write_text("changed synthetic control")
        with patch.object(c, "run_writer") as writer, patch.object(c, "freeze_and_grade", return_value=self.green()) as grade:
            with self.assertRaisesRegex(RuntimeError, "final snapshot hash changed"):
                c.batch(self.root, self.state(), 1)
        writer.assert_not_called()
        grade.assert_not_called()
        self.assertEqual((session / "writer-result.json").read_bytes(), original_record)
        self.assertFalse((session / "trial.json").exists())

    def test_grading_exhaustion_stops_operator_without_writer_failure(self):
        writers = []
        def writer(session, *_args, on_started=None, **kwargs):
            self.grant(session, on_started)
            writers.append(session.name)
            return {"fault": None, "end_reason": "done"}
        with patch.object(h, "prepare", side_effect=self.prepare), patch.object(c, "run_writer", side_effect=writer), \
             patch.object(c, "freeze_and_grade", return_value={"final": {"exit": 2, "infrastructure": True}}) as grade:
            with self.assertRaisesRegex(RuntimeError, "after two retries"): c.batch(self.root, self.state(), 1)
            self.assertEqual(grade.call_count, 3)
            with self.assertRaisesRegex(RuntimeError, "after two retries"): c.batch(self.root, self.state(), 1)
            self.assertEqual(grade.call_count, 3)
        self.assertEqual(writers, ["000-1"])
        self.assertFalse(json.loads((self.root / "slot-000.json").read_text())["complete"])

    def test_stop_during_grading_resumes_frozen_writer(self):
        writers = []
        def writer(session, *_args, on_started=None, **kwargs):
            self.grant(session, on_started)
            writers.append(session.name)
            return {"fault": None, "end_reason": "done"}
        with patch.object(h, "prepare", side_effect=self.prepare), patch.object(c, "run_writer", side_effect=writer), \
             patch.object(c, "freeze_and_grade", side_effect=KeyboardInterrupt):
            with self.assertRaises(KeyboardInterrupt): c.batch(self.root, self.state(), 1)
        with patch.object(c, "run_writer", side_effect=AssertionError("writer must not restart")), \
             patch.object(c, "freeze_and_grade", return_value=self.green()):
            c.batch(self.root, self.state(), 1)
        self.assertEqual(writers, ["000-1"])
        self.assertEqual(json.loads((self.root / "slot-000.json").read_text())["reruns"], 0)

    def test_denied_outside_requests_are_information_but_breaches_stop_run(self):
        use = {"type": "tool_use", "id": "read", "name": "Read", "input": {"file_path": "/outside"}}
        denial = {"type": "system", "subtype": "permission_denied", "tool_name": "Read", "tool_use_id": "read",
                  "decision_reason_type": "other", "decision_reason": c.DENIAL_REASON, "message": "restricted denial"}
        result = {"type": "tool_result", "tool_use_id": "read", "is_error": True, "content": "restricted denial"}
        audit = c.audit([use, denial, result], self.root, g.MODEL)
        self.assertTrue(audit["passed"])
        self.assertEqual(len(audit["denied_outside"]), 1)
        denied_root = self.root / "denied"
        denied_root.mkdir()
        def healthy(session, *_args, on_started=None, **kwargs):
            self.grant(session, on_started)
            return {"fault": None, "end_reason": "done", "audit": audit}
        with patch.object(h, "prepare", side_effect=self.prepare), patch.object(c, "run_writer", side_effect=healthy), \
             patch.object(c, "freeze_and_grade", return_value=self.green()):
            c.batch(denied_root, self.state(), 1)
        trial = json.loads((denied_root / "sessions/000-1/trial.json").read_text())
        self.assertEqual(trial["denied_outside_by_arm"], {"rust": 1})
        breach = c.audit([use], self.root, g.MODEL)
        self.assertTrue(breach["confinement_breach"])
        def writer(session, *_args, on_started=None, **kwargs):
            self.grant(session, on_started)
            g.save(session / "confinement-breach.json", breach)
            raise c.ConfinementBreach("operator stop")
        with patch.object(h, "prepare", side_effect=self.prepare), patch.object(c, "run_writer", side_effect=writer) as launch:
            with self.assertRaises(c.ConfinementBreach): c.batch(self.root, self.state(), 1)
            with self.assertRaises(c.ConfinementBreach): c.batch(self.root, self.state(), 1)
            self.assertEqual(launch.call_count, 1)

    def test_grade_deadline_is_a_solution_timeout(self):
        root = self.root
        (root / "final").mkdir()
        (root / "final/source.rs").write_text("generic control")
        g.save(root / "session.json", {"task": "A2", "arm": "rust", "directory": str(root / "absent")})
        observed = []
        def checks(config, *_args):
            observed.append(config["deadline"])
            return [{"code": 0, "stdout": "", "stderr": ""}]
        with patch.object(sys, "argv", ["harness", "grade", str(root)]), patch.object(h, "STATE", root), \
             patch.object(h, "package", return_value=root), patch.object(h, "checks", side_effect=checks), \
             patch.object(h, "executable_tests", return_value=[]), \
             patch.object(h.time, "monotonic", side_effect=[0, h.GRADE_SECONDS + 1]):
            self.assertEqual(h.main(), 1)
        data = json.loads((root / "final-grade.json").read_text())
        self.assertTrue(data["timed_out"])
        self.assertFalse(data["infrastructure"])
        self.assertEqual(observed, [h.GRADE_SECONDS])
        self.assertLess(h.GRADE_SECONDS, 600)

    def test_harness_owns_the_launch_clock_limit_and_tail_trimming_is_narrow(self):
        g.save(self.root / "session.json", {"dry_run": False, "runs": 0})
        with patch.object(h, "SESSION_SECONDS", 17):
            self.assertEqual(c.launch_clock(self.root)["session_seconds"], 17)
        path = self.root / "stream.jsonl"
        path.write_text('{"type":"system"}\n{"partial":')
        events, tail = c.writer_events(path, True)
        self.assertTrue(tail)
        self.assertEqual(len(events), 1)
        with self.assertRaises(ValueError): c.writer_events(path, False)
        path.write_text('{"type":"system"}\nbad\n{"partial":')
        with self.assertRaises(ValueError): c.writer_events(path, True)
        path.write_bytes(b'{"type":"system"}\n{"partial":"\xe2')
        self.assertEqual(c.writer_events(path, True), ([{"type": "system"}], True))

    def stream_case(self, mode, extra=None, on_started=lambda: None):
        root = self.root / mode
        root.mkdir()
        writer = root / "writer"
        writer.mkdir()
        (writer / "prompt.md").write_text("synthetic control")
        session = root / "session"
        session.mkdir()
        g.save(session / "session.json", {"directory": str(writer), "runs": 20, "started": None,
                                          "dry_run": mode == "time"})
        init = {"type": "system", "subtype": "init", "tools": sorted(g.TOOLS), "plugins": [],
                "mcp_servers": [{"name": "e1", "status": "connected"}], "model": g.MODEL}
        events = root / "events.json"
        g.save(events, [init, *(extra or []), {"type": "result", "subtype": "success", "is_error": False}])
        broker = root / "broker-control.py"
        broker.write_text('''import json, sys, time
from pathlib import Path
path = Path(sys.argv[1], "broker.jsonl")
path.write_text(json.dumps({"event":"start"}) + chr(10))
time.sleep(0.4)
with path.open("a") as log: log.write(json.dumps({"event":"stop"}) + chr(10))
''')
        fixture = root / "cli-control.py"
        fixture.write_text('''import json, subprocess, sys, time
from pathlib import Path
session, events, mode, broker = Path(sys.argv[1]), json.loads(Path(sys.argv[2]).read_text()), sys.argv[3], sys.argv[4]
if mode == "done":
    subprocess.Popen([sys.executable, broker, str(session)], stdin=subprocess.DEVNULL, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    while not (session / "broker.jsonl").exists(): time.sleep(0.01)
print(json.dumps(events[0]), flush=True)
if mode in ("limit","time"):
    sys.stdout.write('{"type":"user","content":"' + "x" * 200000)
    sys.stdout.flush()
    if mode == "limit":
        (session / "budget.json").write_text(json.dumps({"reason":"run_limit","completed_run":20}))
        (session / "broker.jsonl").write_text(json.dumps({"mode":"check","exit":1,"completed_run":20}) + chr(10))
    time.sleep(30)
else:
    for event in events[1:]: print(json.dumps(event), flush=True)
''')
        argv = [sys.executable, str(fixture), str(session), str(events), mode, str(broker)]
        with patch.object(c, "writer_argv", return_value=argv):
            return c.run_writer(session, "unused", g.MODEL, seconds=0.3 if mode == "time" else 3,
                                on_started=on_started)

    def test_placeholder_write_failure_stops_its_granted_guard(self):
        def fail(): raise OSError("cannot save attempt placeholder")
        try:
            with self.assertRaisesRegex(OSError, "placeholder"):
                self.stream_case("limit", on_started=fail)
            self.assertFalse(c.ACTIVE)
        finally:
            c.stop_active()

    def test_native_budget_and_clock_kills_drop_only_the_unterminated_tail(self):
        for mode, reason in [("limit", "run_limit"), ("time", "time_limit")]:
            result = self.stream_case(mode)
            self.assertIsNone(result["fault"], result)
            self.assertEqual(result["end_reason"], reason)
            self.assertTrue(result["truncated_tail"])

    def test_normal_done_waits_for_broker_stop_row(self):
        result = self.stream_case("done")
        self.assertIsNone(result["fault"], result)
        self.assertEqual(result["end_reason"], "done")
        self.assertTrue(any(row.get("event") == "stop" for row in c.jsonl(self.root / "done/session/broker.jsonl")))

    def test_native_undenied_outside_access_stops_for_operator(self):
        use = {"type": "tool_use", "id": "read", "name": "Read", "input": {"file_path": "/outside"}}
        result = {"type": "tool_result", "tool_use_id": "read", "content": "synthetic success", "is_error": False}
        with self.assertRaises(c.ConfinementBreach): self.stream_case("breach", [use, result])
        self.assertTrue((self.root / "breach/session/confinement-breach.json").is_file())

    def test_reused_leader_nonleader_missing_pid_and_ungranted_guard_recover(self):
        root = self.root
        boot = Path("/proc/sys/kernel/random/boot_id").read_text().strip()
        saved = {"pid": 123, "group": 123, "start": "original", "boot": boot}
        g.save(root / "launch.json", {})
        g.save(root / "session.json", {"runs": 0})
        g.save(root / "process.json", saved)
        for current in [dict(saved, start="new"), dict(saved, start="new", group=999)]:
            with patch.object(c, "process_identity", return_value=current), patch.object(c, "group_alive") as group, \
                 patch.object(c.signal, "pidfd_send_signal") as signalled:
                c.reclaim_writer(root)
                group.assert_not_called()
                signalled.assert_not_called()
        with patch.object(c, "process_identity", side_effect=FileNotFoundError): c.reclaim_writer(root)
        (root / "process.json").unlink()
        c.reclaim_writer(root)


class CliCompatTests(unittest.TestCase):
    def test_managed_config_in_any_form_refuses_writer_and_probe_before_launch(self):
        def make_dir(path): path.mkdir()
        def make_file(path): path.write_text("{}")
        def make_dangling(path): path.symlink_to(path.parent / "missing")
        def make_dir_link(path): (path.parent / "real").mkdir(); path.symlink_to(path.parent / "real")
        for make in [make_dir, make_file, make_dangling, make_dir_link]:
            with self.subTest(form=make.__name__), tempfile.TemporaryDirectory() as tmp:
                root = Path(tmp)
                managed = root / "etc/claude-code"
                managed.parent.mkdir()
                session, writer = root / "session", root / "writer"
                session.mkdir(); writer.mkdir(); (writer / "prompt.md").write_text("")
                g.save(session / "session.json", {"directory": str(writer), "started": None, "runs": 0})
                state = {"dry_run": False, "cli": "unused", "model": g.MODEL, "identity": {"cheap": {}}}
                with patch.object(g, "MANAGED_CONFIG", managed), \
                     patch.object(c, "writer_argv", return_value=[sys.executable, "-c", ""]):
                    self.assertEqual(g.managed_config_absent(), {"path": str(managed), "exists": False})
                    make(managed)
                    with self.assertRaisesRegex(ValueError, "claude-code exists"):
                        c.run_writer(session, "unused", g.MODEL)
                    with self.assertRaisesRegex(ValueError, "claude-code exists"):
                        c.probe(root, state)
                self.assertEqual(sorted(p.name for p in session.iterdir()), ["session.json"])
                self.assertFalse((root / "probe-session").exists() or (root / "probe-writer").exists())


if __name__ == "__main__":
    unittest.main()
