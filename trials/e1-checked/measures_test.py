"""Hand-built records only; no toolchains, tasks, suites or reference solutions."""
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch

import measures as m


def save(path, value):
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value))


class MeasuresTest(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.root = Path(self.tmp.name) / "run"
        self.root.mkdir()
        self.order = []
        (self.root / "grader-reviews.md").write_text("Suite removals: none.\n")

    def trial(self, task, arm, rep, runs, first, final, integrity=False):
        slot = len(self.order)
        triple = [task, arm, rep]
        self.order.append(triple)
        session = self.root / "sessions" / f"{slot:03}-1"
        row = {"code": 0 if final else 1, "stdout": "", "stderr": ""}
        grade = {"structure": True, "results": [row]}
        if integrity:
            grade["results"].append({"code": 1, "stage": "integrity"})
        save(session / "final-grade.json", grade)
        save(session / "first-grade.json", {"structure": False, "results": [{"code": 0 if first else 1}]})
        for n in range(1, runs + 1):
            save(session / f"snapshot-{n:02}" / "results.json", [
                {"code": 1 if n == 1 else 0, "command": ["test", "--exact", "tests::edge"],
                 "stderr": "error[E0308]: private source\nwarning: private source\n  = note: `#[warn(unused_mut)]` on by default\nFX0301",
                 "stdout": "private test body"}])
        (session / "writer.jsonl").write_text("\n".join(json.dumps(e) for e in [
            {"type": "result", "usage": {k: 9999 for k in m.TOKEN_KEYS}},
            {"type": "assistant", "message": {"id": "a", "usage": {"output_tokens": 1}}},
            {"type": "result", "num_turns": 7, "usage": dict(zip(m.TOKEN_KEYS, [11, 22, 33, 44]))}
        ]) + "\n")
        (session / "final").mkdir()
        (session / "final" / "source.rs").write_text("// sync_all(\npub fn f() {}\n")
        attempt = {"attempt": 1, "session": str(session), "runs": runs, "fault": None,
                   "wall_seconds": 12, "first_snapshot": {"present": runs > 0, "bytes": 42 if runs else 0, "holes": 2 if runs else 0},
                   "audit": {"turns": 1, "peak_context": 80, "reference_reads": [
                       {"document": "reference.md", "sections": ["1", "1.2"], "offset": 1}],
                       "tokens": {k: 999 for k in m.TOKEN_KEYS}, "errors": [], "denied_outside": [], "confinement_breach": []}}
        record = {"slot": slot, "triple": triple, "attempts": [attempt], "complete": True,
                  "reruns": 0, "failed_infrastructure": False, "first_pass": True, "final_pass": True}
        save(self.root / f"slot-{slot:03}.json", record)
        save(self.root / "state.json", {"order": self.order, "dry_run": False})
        return slot, record, session

    def fixture(self):
        # Known first: Rust F=P=0, I undefined, failure-penalized mean=20.
        # Tide F=2/4, P=3/4, passing I=(1+3+2)/3=2, all-trial I=26/4.
        for task in ["A1", "A2"]:
            for rep in [1, 2]:
                self.trial(task, "rust", rep, 1, False, False)
                self.trial(task, "tide", rep, (1 if rep == 1 else 3) if task == "A1" else (2 if rep == 1 else 0),
                           task == "A1", task == "A1" or rep == 1)

    def test_fixture_and_cli(self):
        self.fixture()
        measures, trials = m.measure(self.root, self.root / "grader-reviews.md")
        self.assertEqual(measures["arms"]["rust"], {"trials": 4, "first_passes": 0, "final_passes": 0,
                         "F": 0, "P": 0, "I": None, "all_trials_I_failures_as_20": 20})
        self.assertEqual(measures["arms"]["tide"], {"trials": 4, "first_passes": 2, "final_passes": 3,
                         "F": .5, "P": .75, "I": 2, "all_trials_I_failures_as_20": 6.5})
        self.assertEqual([t["met"] for t in measures["thresholds"]], [None, None, None])
        self.assertIsNone(measures["all_thresholds_met"])
        self.assertFalse(measures["sample"]["protocol_sample"])
        intervals = measures["bootstrap"]["intervals"]
        for metric in ["F", "P", "I"]:
            self.assertEqual(intervals[metric]["ratio"]["undefined_resamples"], 10000)
            self.assertIsNone(intervals[metric]["ratio"]["lower"])
        self.assertEqual(intervals["F"]["difference"]["width"], 1)
        self.assertEqual(intervals["P"]["difference"]["width"], .5)
        self.assertEqual(intervals["I"]["difference"]["undefined_resamples"], 10000)
        self.assertFalse(trials[-1]["first_pass"])
        attempt = trials[1]["attempts"][0]
        self.assertEqual(attempt["tokens"], dict(zip(m.TOKEN_KEYS, [11, 22, 33, 44])))
        self.assertEqual(attempt["turns"], 1)
        self.assertEqual(attempt["reference_sections"], {"reference.md": ["1", "1.2"]})
        self.assertEqual(attempt["runs"][0]["diagnostic_codes"], ["E0308", "FX0301", "unused_mut"])
        self.assertEqual(attempt["runs"][0]["failing_tests"], ["tests::edge"])
        self.assertNotIn("private", json.dumps([measures, trials]))
        out = Path(self.tmp.name) / "out"
        command = [sys.executable, str(Path(m.__file__)), str(self.root), str(out)]
        run = subprocess.run(command, capture_output=True, text=True, timeout=30)
        self.assertEqual(run.returncode, 0, run.stderr)
        self.assertEqual(json.loads((out / "measures.json").read_text()), measures)
        self.assertEqual(json.loads((out / "per-trial.json").read_text()), trials)
        self.assertNotEqual(subprocess.run(command, capture_output=True, timeout=30).returncode, 0)
        command[-1] = str(self.root / "out")
        self.assertNotEqual(subprocess.run(command, capture_output=True, timeout=30).returncode, 0)
        self.assertFalse((self.root / "out").exists())

    def test_reviews_counts_and_missing_usage(self):
        slot, record, session = self.trial("B9", "rust", 1, 1, True, True, integrity=True)
        self.trial("B9", "tide", 1, 1, True, True)
        self.trial("B9", "rust", 2, 1, True, True)
        self.trial("B9", "tide", 2, 1, True, True)
        review = self.root / "grader-reviews.md"
        review.write_text("Suite removals: none.\n## slot 000 B9/rust/1\nCONCURRENCY: n/a\nINTEGRITY: STANDS\n")
        measures, trials = m.measure(self.root, review)
        self.assertTrue(trials[0]["final_pass"])
        self.assertEqual(measures["counts"]["rust"]["structure_comment_only_trials"], 2)
        self.assertEqual(measures["bootstrap"]["intervals"]["P"]["ratio"]["width"], 0)
        review.write_text(review.read_text().replace("STANDS", "DOES-NOT-STAND"))
        self.assertFalse(m.measure(self.root, review)[1][0]["final_pass"])
        review.write_text(review.read_text().replace("DOES-NOT-STAND", "n/a"))
        with self.assertRaisesRegex(ValueError, "integrity"):
            m.measure(self.root, review)
        review.write_text("Suite removals: none.\n## slot 000 B9/rust/1\nCONCURRENCY: FAIL\nINTEGRITY: STAND\n")
        with self.assertRaisesRegex(ValueError, "integrity"):
            m.measure(self.root, review)
        review.write_text(review.read_text().replace("INTEGRITY: STAND\n", "INTEGRITY: STANDS\n"))
        self.assertFalse(m.measure(self.root, review)[1][0]["final_pass"])
        record["attempts"][0]["audit"].update(errors=["forbidden tool: Bash"], denied_outside=[{}], confinement_breach=[{}])
        record.update(reruns=1, failed_infrastructure=True)
        record["attempts"] = [dict(record["attempts"][0], fault="cli_error"), dict(record["attempts"][0], attempt=2, fault="cli_error")]
        save(self.root / f"slot-{slot:03}.json", record)
        (session / "writer.jsonl").write_text("")
        measures, trials = m.measure(self.root, review)
        self.assertEqual(measures["counts"]["rust"]["infrastructure_reruns"], 1)
        self.assertEqual(measures["counts"]["rust"]["second_fault_failures"], 1)
        self.assertEqual(measures["counts"]["rust"]["tools_outside_set"], 2)
        self.assertEqual(measures["counts"]["rust"]["denied_outside_requests"], 2)
        self.assertEqual(measures["counts"]["rust"]["confinement_breaches"], 2)
        self.assertIsNone(trials[0]["attempts"][0]["tokens"])
        self.assertTrue(measures["unavailable"])

    def test_diagnostics_and_unnamed_lints(self):
        row = {"code": 1, "stderr": (
            "/w/src/main.rs:98:4: warning: function `run` is never used\n"
            "/w/src/main.rs:99:4: warning: something [FX0508]\n"
            "/w/src/main.rs:100:4: warning[E0308]: something\n"
            "  = note: `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default\n"
            "  = note: `#[deny(unused_mut)]` on by default\n"
            "  = note: `#[allow(unused_variables)]` on by default\n"
            "1 | #[allow(quoted_allow)]\n2 | #[warn(quoted_warn)]\n"
            '3 | "#[deny(quoted_deny)]"\n'),
            "stdout": "test src/lib.rs - parse (line 3) ... FAILED\n"}
        summary = m.summarize_rows([row])
        self.assertEqual(summary["diagnostic_codes"], ["E0308", "FX0508", "dead_code", "unused_mut"])
        self.assertEqual(summary["unnamed_lint_warnings"], 1)
        self.assertEqual(summary["failing_tests"], ["src/lib.rs - parse (line 3)"])
        self.fixture()
        session = self.root / "sessions" / "001-1"
        save(session / "snapshot-01" / "results.json", [row])
        save(session / "first-grade.json", {"structure": False, "results": [row]})
        save(session / "final-grade.json", {"structure": False, "results": [row]})
        measures, trials = m.measure(self.root, self.root / "grader-reviews.md")
        self.assertEqual(trials[1]["attempts"][0]["runs"][0]["unnamed_lint_warnings"], 1)
        for phase in ("first", "final"):
            self.assertEqual(trials[1]["grades"][phase]["unnamed_lint_warnings"], 1)
        self.assertEqual({r["field"] for r in measures["unavailable"]},
                         {"run_1.lint_names", "first_grade.lint_names", "final_grade.lint_names"})
        self.assertTrue(all(r["slot_attempt"] == [1, 1] and "--error-format=short" in r["reason"]
                            for r in measures["unavailable"]))
        self.assertIn("--error-format=short", measures["definitions"]["unnamed_lint_warnings"])

    def test_unattached_session_breach_file(self):
        self.fixture()
        save(self.root / "sessions" / "unrecorded" / "confinement-breach.json", {"confinement_breach": [{}]})
        measures, _ = m.measure(self.root, self.root / "grader-reviews.md")
        self.assertEqual(measures["session_confinement_breach_files"], 1)
        self.assertEqual([measures["counts"][arm]["confinement_breaches"] for arm in ("rust", "tide")], [0, 0])

    def test_comment_mask_preserves_literals_and_nested_comments(self):
        source = 'r##"// literal /*"##; "/* literal */"; \'/\'; // gone\n/* outer /* inner */ end */ x'
        masked = m.without_comments(source)
        self.assertIn('r##"// literal /*"##', masked)
        self.assertIn('"/* literal */"', masked)
        self.assertNotIn("gone", masked)
        self.assertNotIn("inner", masked)
        self.assertEqual(masked.count("\n"), source.count("\n"))

    def test_exact_threshold_boundaries(self):
        order = [[f"A{task}", arm, rep] for task in range(1, 31)
                 for arm in ("rust", "tide") for rep in (1, 2)]
        save(self.root / "state.json", {"order": order, "dry_run": False})
        # F: 20/60 = (4/5)*(25/60); I: 17/3 = (3/2)*(34/9).
        for tide_first, tide_runs, expected in [(20, [5, 6, 6], [True, False, True]),
                                                (19, [5, 6, 6], [False, False, True]),
                                                (20, [6, 6, 6], [True, False, False]),
                                                (20, [4, 4, 4, 4, 4, 4, 4, 3, 3], [True, True, True]),
                                                (20, [], [True, False, False])]:
            with self.subTest(tide_first=tide_first, tide_runs=tide_runs):
                trials = []
                for arm, first, runs in [("tide", tide_first, tide_runs), ("rust", 25, [4]*7 + [3, 3])]:
                    trials.extend({"arm": arm, "first_pass": i < first, "final_pass": i < len(runs),
                                   "iterations": runs[i] if i < len(runs) else 0,
                                   "structure_comment_only": False, "infrastructure_reruns": 0,
                                   "second_fault_failure": False, "attempts": []} for i in range(60))
                with patch.object(m, "read_trials", return_value=(trials, [])), patch.object(m, "bootstrap", return_value={}):
                    measures, _ = m.measure(self.root, self.root / "grader-reviews.md")
                self.assertEqual([t["met"] for t in measures["thresholds"]], expected)

    def fault_fixture(self, fault):
        slot, record, session = self.trial("A1", "tide", 1, 2, True, True)
        for arm, rep in [("rust", 1), ("rust", 2), ("tide", 2)]:
            self.trial("A1", arm, rep, 1, True, True)
        retry_session = session.with_name("000-2")
        session.rename(retry_session)
        session.mkdir()
        retry = dict(record["attempts"][0], session=str(retry_session), attempt=2)
        record.update(reruns=1, attempts=[dict(fault, session=str(session), attempt=1, dry_run=False), retry])
        save(self.root / f"slot-{slot:03}.json", record)
        return record, session, retry_session

    def test_malformed_writer_fault_then_retry(self):
        # classify_session: malformed JSON yields a partial audit and log_audit fault.
        _, session, _ = self.fault_fixture({"fault": "log_audit", "end_reason": "log_audit", "runs": 1,
            "audit": {"passed": False, "errors": ["invalid JSON"]}, "truncated_tail": False})
        (session / "writer.jsonl").write_text('{"type": "result",\n')
        save(session / "snapshot-01" / "results.json", [{"code": 1, "stderr": "E0308"}])
        measures, trials = m.measure(self.root, self.root / "grader-reviews.md")
        self.assertTrue(trials[0]["first_pass"] and trials[0]["final_pass"])
        self.assertEqual(trials[0]["iterations"], 2)
        self.assertEqual(trials[0]["attempts"][0]["runs"][0]["diagnostic_codes"], ["E0308"])
        self.assertIsNone(trials[0]["attempts"][0]["tokens"])
        self.assertIsNone(trials[0]["attempts"][0]["reference_sections"])
        self.assertIsNone(measures["counts"]["tide"]["denied_outside_requests"])
        self.assertEqual(measures["counts"]["tide"]["infrastructure_reruns"], 1)
        self.assertFalse(trials[0]["second_fault_failure"])
        self.assertTrue(measures["unavailable"])

    def test_interrupted_attempt_then_retry(self):
        # batch's recovery placeholder has no runs, audit, wall time or snapshots.
        _, session, _ = self.fault_fixture({"fault": "host_interruption", "end_reason": "host_interruption",
                                           "evidence": "unfinished granted writer"})
        # session.json can recover the run count even when its last result was not saved.
        save(session / "session.json", {"runs": 1})
        measures, trials = m.measure(self.root, self.root / "grader-reviews.md")
        self.assertTrue(trials[0]["first_pass"] and trials[0]["final_pass"])
        self.assertEqual(trials[0]["iterations"], 2)
        self.assertIsNone(trials[0]["attempts"][0]["runs"][0]["verdict"])
        self.assertIsNone(trials[0]["attempts"][0]["turns"])
        self.assertTrue(any(row["field"] == "run_1" for row in measures["unavailable"]))

    def test_retained_second_interruption_fails(self):
        record, _, retry_session = self.fault_fixture({"fault": "host_interruption", "end_reason": "host_interruption"})
        record["attempts"][1] = dict(record["attempts"][0], attempt=2, session=str(retry_session))
        record["failed_infrastructure"] = True
        (retry_session / "writer.jsonl").unlink()
        save(self.root / "slot-000.json", record)
        measures, trials = m.measure(self.root, self.root / "grader-reviews.md")
        self.assertFalse(trials[0]["first_pass"] or trials[0]["final_pass"])
        self.assertIsNone(trials[0]["iterations"])
        self.assertIsNone(trials[0]["attempts"][1]["runs"])
        self.assertEqual(measures["counts"]["tide"]["second_fault_failures"], 1)
        self.assertEqual(measures["arms"]["tide"]["I"], 1)
        self.assertEqual(measures["arms"]["tide"]["all_trials_I_failures_as_20"], 10.5)
        self.assertTrue(any(row["field"] == "runs" for row in measures["unavailable"]))

    def test_successful_attempt_still_rejects_inconsistent_records(self):
        _, record, session = self.trial("A1", "rust", 1, 1, True, True)
        attempt = record["attempts"][0]
        writer = session / "writer.jsonl"
        original = writer.read_text()
        writer.write_text("{bad JSON\n")
        with self.assertRaises(ValueError):
            m.read_attempt(self.root, attempt, "rust", [], [0, 1])
        writer.unlink()
        with self.assertRaises(OSError):
            m.read_attempt(self.root, attempt, "rust", [], [0, 1])
        writer.write_text(original)
        del attempt["runs"]
        with self.assertRaises(KeyError):
            m.read_attempt(self.root, attempt, "rust", [], [0, 1])
        attempt["runs"] = 1
        (session / "snapshot-01" / "results.json").unlink()
        with self.assertRaises(OSError):
            m.read_attempt(self.root, attempt, "rust", [], [0, 1])

    def test_paired_bootstrap_counts_undefined_resamples(self):
        trials = [{"task": task, "arm": arm, "replicate": rep,
                   "first_pass": task == "A1", "final_pass": task == "A1" or arm == "tide",
                   "iterations": 3 if task == "A2" and arm == "tide" else 1}
                  for task in ["A1", "A2"] for arm in ["rust", "tide"] for rep in [1, 2]]
        intervals = m.bootstrap(trials)
        # Seed 20261031 draws [A2, A2] 2507 times: only these lose Rust's denominator.
        for metric in ["F", "P", "I"]:
            self.assertEqual(intervals[metric]["ratio"]["undefined_resamples"], 2507)
        self.assertEqual(intervals["I"]["difference"]["undefined_resamples"], 2507)
        self.assertEqual(intervals["F"]["difference"]["width"], 0)
        self.assertEqual(intervals["I"]["ratio"]["lower"], 1)
        self.assertEqual(intervals["I"]["ratio"]["upper"], 2)


if __name__ == "__main__":
    unittest.main()
