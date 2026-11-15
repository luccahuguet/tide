#!/usr/bin/env python3
"""Reproduce E1 protocol §7–§8 from stored records, without running writer code.

Usage: python3 trials/e1-checked/measures.py RUN_DIR NEW_OUTPUT_DIR
       [--grader-reviews FILE]

Writes measures.json and per-trial.json. The output directory must be new and
outside RUN_DIR. Only names, codes, counts and measurement metadata leave the
reader; prompts, diagnostic prose, source and test bodies are never emitted.
Suite removals other than the recorded 'none' require regrading first.
Threshold verdicts are null (not applicable) outside the 30-task paired sample.
"""
import argparse
from collections import defaultdict
from fractions import Fraction
from hashlib import sha256
import json
from pathlib import Path
import random
import re

import harness

TOKEN_KEYS = ("input_tokens", "cache_read_input_tokens", "cache_creation_input_tokens", "output_tokens")
CONCURRENCY_TASKS = {"C1", "C2", "C3", "C4", "C5", "C6", "C9"}
SEED = 20261031
RESAMPLES = 10000
LINT_NAMES_UNAVAILABLE = "Short-format warnings omit lint names and notes; tide-admit forces --error-format=short."


def load(path):
    return json.loads(path.read_text())


def reviews(path, order):
    text = path.read_text()
    if not re.search(r"Suite removals:\s*none\.", text, re.I):
        raise ValueError("suite removals must be recorded as none; regrade removals before measuring")
    result = {}
    for block in re.split(r"(?m)^## slot ", text)[1:]:
        head = re.match(r"(\d+) ([A-C]\d+)/(rust|tide)/([12])\s*\n", block)
        if not head:
            raise ValueError("invalid grader-review header")
        slot, task, arm, rep = head.groups()
        slot = int(slot)
        if slot >= len(order) or order[slot] != [task, arm, int(rep)] or slot in result:
            raise ValueError("grader-review slot identity mismatch")
        verdicts = {}
        for kind, allowed in [("CONCURRENCY", {"PASS", "FAIL", "n/a"}),
                              ("INTEGRITY", {"STANDS", "DOES-NOT-STAND", "n/a"})]:
            values = re.findall(rf"(?m)^{kind}: ([^\r\n]+)$", block)
            if len(values) != 1 or values[0] not in allowed:
                raise ValueError("invalid or missing " + kind.lower() + " verdict")
            verdicts[kind.lower()] = values[0]
        result[slot] = verdicts
    return result


def without_comments(text):
    """Mask Rust comments, retaining literals and line breaks for literal searches."""
    token = re.compile(r'r(?P<hashes>\#*)"[\s\S]*?"(?P=hashes)|"(?:\\[\s\S]|[^"\\])*"'
                       r"|'(?:\\(?:u\{[0-9a-fA-F_]+\}|x[0-9a-fA-F]{2}|.)|[^'\\\n])'"
                       r"|(?P<line>//[^\n]*)|(?P<block>/\*)")
    pieces, offset = [], 0
    while match := token.search(text, offset):
        pieces.append(text[offset:match.start()])
        end = match.end()
        if match.group("block"):
            depth = 1
            for boundary in re.finditer(r"/\*|\*/", text[end:]):
                depth += 1 if boundary[0] == "/*" else -1
                if depth == 0:
                    end += boundary.end()
                    break
            else:
                end = len(text)
        value = text[match.start():end]
        pieces.append(re.sub(r"[^\n]", " ", value) if match.group("line") or match.group("block") else value)
        offset = end
    return "".join(pieces) + text[offset:]


def summarize_rows(rows):
    """Extract identifiers only; exact test commands also name crashes/timeouts."""
    codes, failing = set(), set()
    unnamed_lints = 0
    for row in rows:
        for field in ("stdout", "stderr"):
            text = row.get(field, "")
            codes.update(re.findall(r"\b(?:FX|E)\d{4}\b", text))
            codes.update(re.findall(
                r"(?m)^[ \t]*= note: `#\[(?:warn|deny|forbid)\(([a-z][a-z0-9_]*(?:::[a-z][a-z0-9_]*)*)\)\]`"
                r"(?: \(part of `[^`\r\n]+`\))? on by default$", text))
            unnamed_lints += len(re.findall(
                r"(?m)^\S+:\d+:\d+: warning: (?![^\r\n]*\[(?:FX|E)\d{4}\])[^\r\n]+$", text))
            for line in text.splitlines():
                try:
                    event = json.loads(line)
                except ValueError:
                    continue
                if isinstance(event, dict) and event.get("reason") == "compiler-message":
                    code = (event.get("message", {}).get("code") or {}).get("code")
                    if isinstance(code, str):
                        codes.add(code)
            failing.update(re.findall(r"(?m)^test (.+?) \.\.\. FAILED$", text))
        if row["code"] != 0:
            command = row.get("command", [])
            if "--exact" in command:
                failing.add(command[command.index("--exact") + 1])
            if "case" in row:
                failing.add("case" + str(row["case"]))
    return {"verdict": "PASS" if rows and all(r["code"] == 0 for r in rows) else "FAIL",
            "exit_codes": [r["code"] for r in rows], "diagnostic_codes": sorted(codes),
            "unnamed_lint_warnings": unnamed_lints, "failing_tests": sorted(failing)}


def read_attempt(root, attempt, arm, unavailable, identity):
    session = Path(attempt["session"]).resolve()
    if not session.is_relative_to(root / "sessions"):
        raise ValueError("session must be inside run/sessions")
    fault = bool(attempt.get("fault"))
    usage = None
    token_reason = "last result event lacks four token totals"
    try:
        with (session / "writer.jsonl").open() as stream:
            for line in stream:
                try:
                    event = json.loads(line)
                except ValueError:
                    # A killed writer may leave a truncated final JSONL event.
                    if attempt.get("truncated_tail") and not line.endswith("\n"):
                        continue
                    raise ValueError("invalid writer event") from None
                if isinstance(event, dict) and event.get("type") == "result":
                    usage = event.get("usage")
    except (OSError, ValueError):
        if not fault:
            raise
        usage = None
        token_reason = "writer log missing or malformed in faulted attempt"
    tokens = {key: usage.get(key) for key in TOKEN_KEYS} if isinstance(usage, dict) else None
    if tokens is None or any(type(value) is not int or value < 0 for value in tokens.values()):
        tokens = None
        unavailable.append({"slot_attempt": identity, "field": "tokens", "reason": token_reason})
    audit = attempt.get("audit", {})
    sections = defaultdict(set)
    if arm == "tide":
        for read in audit.get("reference_reads", []):
            sections[read["document"]].update(read["sections"])
    run_count = attempt.get("runs") if fault else attempt["runs"]
    if fault and run_count is None:
        try:
            run_count = load(session / "session.json")["runs"]
        except (OSError, ValueError, KeyError, TypeError):
            pass
    if type(run_count) is not int or run_count < 0:
        if not fault:
            raise ValueError("invalid run count")
        run_count = None
    runs = [] if run_count is not None else None
    for number in range(1, (run_count or 0) + 1):
        try:
            summary = summarize_rows(load(session / f"snapshot-{number:02}" / "results.json"))
        except (OSError, ValueError, KeyError, TypeError):
            if not fault:
                raise
            summary = dict.fromkeys(("verdict", "exit_codes", "diagnostic_codes", "unnamed_lint_warnings", "failing_tests"))
            unavailable.append({"slot_attempt": identity, "field": f"run_{number}",
                                "reason": "run results missing or malformed in faulted attempt"})
        if summary["unnamed_lint_warnings"]:
            unavailable.append({"slot_attempt": identity, "field": f"run_{number}.lint_names",
                                "reason": LINT_NAMES_UNAVAILABLE})
        runs.append({"number": number, **summary})
    result = {"attempt": attempt["attempt"], "fault": attempt.get("fault"), "runs": runs,
              "wall_seconds": attempt.get("wall_seconds"), "turns": audit.get("turns"),
              "tokens": tokens, "peak_context": audit.get("peak_context"),
              "reference_sections": ({doc: sorted(values) for doc, values in sorted(sections.items())}
                                     if arm != "tide" or "reference_reads" in audit else None),
              "first_snapshot": attempt.get("first_snapshot"),
              # A classifier error audit has only passed/errors, without tool observations.
              "tools_outside_set": (sum(error.startswith("forbidden tool: ") for error in audit["errors"])
                                    if "turns" in audit and "errors" in audit else None),
              "denied_outside_requests": len(audit["denied_outside"]) if "denied_outside" in audit else None,
              "confinement_breaches": len(audit["confinement_breach"]) if "confinement_breach" in audit else None}
    for field in ("runs", "wall_seconds", "turns", "peak_context", "first_snapshot", "reference_sections",
                  "tools_outside_set", "denied_outside_requests", "confinement_breaches"):
        if result[field] is None:
            unavailable.append({"slot_attempt": identity, "field": field, "reason": "not recorded by coordinator"})
    return result, session


def read_trials(root, state, decisions):
    trials, unavailable = [], []
    for slot, triple in enumerate(state["order"]):
        task, arm, rep = triple
        record = load(root / f"slot-{slot:03}.json")
        if record["triple"] != triple or record["slot"] != slot or not record["complete"]:
            raise ValueError("incomplete or mismatched slot record")
        attempts = []
        for attempt in record["attempts"]:
            safe, session = read_attempt(root, attempt, arm, unavailable, [slot, attempt["attempt"]])
            attempts.append(safe)
        if not attempts:
            raise ValueError("completed slot has no attempt")
        last = record["attempts"][-1]
        review = decisions.get(slot, {"concurrency": "n/a", "integrity": "n/a"})
        first_pass = final_pass = comment_only = False
        grades = {}
        if not last.get("fault"):
            for phase in ("first", "final"):
                grade = load(session / f"{phase}-grade.json")
                if grade.get("infrastructure") or any(r.get("infrastructure") for r in grade["results"]):
                    raise ValueError("unresolved grading infrastructure failure")
                grades[phase] = summarize_rows(grade["results"])
                if grades[phase]["unnamed_lint_warnings"]:
                    unavailable.append({"slot_attempt": [slot, last["attempt"]], "field": f"{phase}_grade.lint_names",
                                        "reason": LINT_NAMES_UNAVAILABLE})
                if phase == "first":
                    # The first grade compiles the writer's tests and the library suite.
                    first_pass = last["runs"] > 0 and grades[phase]["verdict"] == "PASS" and not grade.get("timed_out")
                    continue
                integrity_rows = any(r.get("stage") == "integrity" for r in grade["results"])
                if integrity_rows and review["integrity"] == "n/a":
                    raise ValueError("missing integrity review")
                if task in CONCURRENCY_TASKS and review["concurrency"] == "n/a":
                    raise ValueError("missing concurrency review")
                rows = [r for r in grade["results"] if r.get("stage") != "integrity"]
                final_pass = (bool(rows) and all(r["code"] == 0 for r in rows) and grade["structure"]
                              and not grade.get("timed_out") and review["concurrency"] != "FAIL"
                              and review["integrity"] != "DOES-NOT-STAND")
                if grade["structure"]:
                    text = (session / "final" / "source.rs").read_text()
                    comment_only = not harness.structure(task, arm, without_comments(text))
        trials.append({"slot": slot, "task": task, "arm": arm, "replicate": rep,
                       "first_pass": first_pass, "final_pass": final_pass,
                       "iterations": len(attempts[-1]["runs"]) if attempts[-1]["runs"] is not None else None,
                       "review": review, "grades": grades, "structure_comment_only": bool(comment_only),
                       "infrastructure_reruns": record["reruns"], "second_fault_failure": record["failed_infrastructure"],
                       "attempts": attempts})
    return trials, unavailable


def arm_measures(trials):
    result = {}
    for arm in ("rust", "tide"):
        records = [t for t in trials if t["arm"] == arm]
        passing = [t for t in records if t["final_pass"]]
        n = len(records)
        first = sum(t["first_pass"] for t in records)
        result[arm] = {"trials": n, "first_passes": first, "final_passes": len(passing),
                       "F": first / n, "P": len(passing) / n,
                       "I": sum(t["iterations"] for t in passing) / len(passing) if passing else None,
                       "all_trials_I_failures_as_20": sum(t["iterations"] if t["final_pass"] else 20 for t in records) / n}
    return result


def comparisons(arms, metric):
    tide, rust = arms["tide"][metric], arms["rust"][metric]
    return {"ratio": tide / rust if tide is not None and rust not in (None, 0) else None,
            "difference": tide - rust if tide is not None and rust is not None else None}


def interval(values):
    defined = sorted(v for v in values if v is not None)
    def percentile(p):
        index = (len(defined) - 1) * p
        lo = int(index)
        return defined[lo] + (defined[min(lo + 1, len(defined) - 1)] - defined[lo]) * (index - lo)
    lower, upper = (percentile(.025), percentile(.975)) if defined else (None, None)
    return {"lower": lower, "upper": upper, "width": upper - lower if defined else None,
            "defined_resamples": len(defined), "undefined_resamples": len(values) - len(defined)}


def bootstrap(trials):
    tasks = sorted({t["task"] for t in trials})
    grouped = {task: [t for t in trials if t["task"] == task] for task in tasks}
    samples = {metric: {kind: [] for kind in ("ratio", "difference")} for metric in ("F", "P", "I")}
    rng = random.Random(SEED)
    for _ in range(RESAMPLES):
        arms = arm_measures([t for task in rng.choices(tasks, k=len(tasks)) for t in grouped[task]])
        for metric, kinds in samples.items():
            for kind, value in comparisons(arms, metric).items():
                kinds[kind].append(value)
    return {metric: {kind: interval(values) for kind, values in kinds.items()} for metric, kinds in samples.items()}


def measure(root, review_file):
    root = root.resolve()
    state = load(root / "state.json")
    if state.get("dry_run"):
        raise ValueError("a dry run cannot supply gate measures")
    order = state["order"]
    if not order or any(len(t) != 3 or t[1] not in ("rust", "tide") or t[2] not in (1, 2) for t in order):
        raise ValueError("invalid trial order")
    tasks = sorted({t[0] for t in order})
    expected = sorted([task, arm, rep] for task in tasks for arm in ("rust", "tide") for rep in (1, 2))
    if sorted(order) != expected:
        raise ValueError("each task must have both replicates and both arms")
    protocol_sample = len(tasks) == 30
    trials, unavailable = read_trials(root, state, reviews(review_file, order))
    arms = arm_measures(trials)
    tide, rust = arms["tide"], arms["rust"]
    exact = {arm: {"F": Fraction(values["first_passes"], values["trials"]),
                   "P": Fraction(values["final_passes"], values["trials"]),
                   "I": Fraction(sum(t["iterations"] for t in trials if t["arm"] == arm and t["final_pass"]),
                                 values["final_passes"]) if values["final_passes"] else None}
             for arm, values in arms.items()}
    thresholds = [{"inequality": "F(Tide) >= 0.8 * F(Rust)", "left": tide["F"], "right": .8 * rust["F"],
                   "met": exact["tide"]["F"] >= Fraction(4, 5) * exact["rust"]["F"]},
                  {"inequality": "P(Tide) >= P(Rust)", "left": tide["P"], "right": rust["P"],
                   "met": exact["tide"]["P"] >= exact["rust"]["P"]},
                  {"inequality": "I(Tide) <= 1.5 * I(Rust)", "left": tide["I"],
                   "right": 1.5 * rust["I"] if rust["I"] is not None else None,
                   "met": exact["tide"]["I"] is not None and exact["rust"]["I"] is not None
                          and exact["tide"]["I"] <= Fraction(3, 2) * exact["rust"]["I"]}]
    if not protocol_sample:
        for threshold in thresholds:
            threshold["met"] = None
    counts = {}
    for arm in ("rust", "tide"):
        records = [t for t in trials if t["arm"] == arm]
        counts[arm] = {"structure_comment_only_trials": sum(t["structure_comment_only"] for t in records),
                       "infrastructure_reruns": sum(t["infrastructure_reruns"] for t in records),
                       "second_fault_failures": sum(t["second_fault_failure"] for t in records)}
        for field in ("tools_outside_set", "denied_outside_requests", "confinement_breaches"):
            values = [a[field] for t in records for a in t["attempts"]]
            counts[arm][field] = sum(values) if all(v is not None for v in values) else None
    return {"schema": 1, "sample": {"tasks": len(tasks), "trials_per_arm": len(order) // 2,
                                    "protocol_sample": protocol_sample},
            "definitions": {"turns": "Distinct assistant message IDs, coordinator.audit turns; not CLI result.num_turns.",
                            "tokens": "All four cumulative totals from each session's last type=result event in writer.jsonl; no audit-token fallback.",
                            "iterations": "All check/test runs in the retained attempt, including runs after green; rerun attempts are recorded separately.",
                            "structure_comment_only_trials": "Trials whose recorded structure check passes but harness.structure fails after masking comments, retaining string literals.",
                            "unnamed_lint_warnings": "Count of short-format warning lines without a diagnostic code, per run or grade. " + LINT_NAMES_UNAVAILABLE,
                            "session_confinement_breach_files": "Count of sessions/*/confinement-breach.json files, including sessions absent from slot records; separate from per-arm audit-event counts, without adding duplicates.",
                            "thresholds": "Verdicts (met and all_thresholds_met) are null, meaning not applicable, unless the sample has 30 tasks with both replicates and both arms.",
                            "ratio": "Tide / Rust", "difference": "Tide - Rust"},
            "grader_reviews_sha256": sha256(review_file.read_bytes()).hexdigest(),
            "suite_removals": 0, "arms": arms, "thresholds": thresholds,
            "all_thresholds_met": all(t["met"] for t in thresholds) if protocol_sample else None,
            "session_confinement_breach_files": len(list(root.glob("sessions/*/confinement-breach.json"))),
            "point_comparisons": {metric: comparisons(arms, metric) for metric in ("F", "P", "I")},
            "bootstrap": {"information_only": True, "seed": SEED, "resamples": RESAMPLES,
                          "unit": "task, both replicates and arms together; random.Random(seed).choices(sorted_tasks, k=task_count)",
                          "percentiles": [2.5, 97.5], "interpolation": "linear, index (n - 1) * p",
                          "undefined": "Omitted from percentiles and counted per interval; all undefined yields null bounds and width.",
                          "intervals": bootstrap(trials)}, "counts": counts, "unavailable": unavailable}, trials


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("run_dir", type=Path)
    parser.add_argument("output_dir", type=Path)
    parser.add_argument("--grader-reviews", type=Path)
    args = parser.parse_args()
    root, output = args.run_dir.resolve(), args.output_dir.resolve()
    if output.is_relative_to(root) or root.is_relative_to(output) or output.exists():
        parser.error("output must be a new directory outside the run directory")
    try:
        measures, trials = measure(root, args.grader_reviews or root / "grader-reviews.md")
        output.mkdir(parents=True)
        for name, value in (("measures.json", measures), ("per-trial.json", trials)):
            (output / name).write_text(json.dumps(value, indent=2, sort_keys=True, allow_nan=False) + "\n")
    except (OSError, ValueError, KeyError, TypeError):
        # Record contents may be held-out material, including JSON parser error snippets.
        parser.exit(2, "unreadable or inconsistent measurement records; no source details emitted\n")
    print("Wrote measures.json and per-trial.json")


if __name__ == "__main__":
    main()
