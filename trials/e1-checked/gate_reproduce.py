#!/usr/bin/env python3
"""Unpack E1 records, or regrade a fixed sample in a coordinator-booked slot.

unpack DEST: verifies the archive and members, then relocates slot session paths.
regrade RECORDS GATE_WORKTREE NEW_OUTPUT: runs the frozen harness, never writers.
The latter compiles: use the gate-report.md preflight and heavy.sh command.
"""
import argparse
from hashlib import sha256
import json
import os
from pathlib import Path
import random
import shutil
import subprocess
import sys
import tarfile

HERE = Path(__file__).resolve().parent
REVISION = "675c26e126df6db6090a94ddde6acc8004fe454d"
SEED = 20261031


def load(path):
    return json.loads(path.read_text())


def unpack(destination):
    archive = HERE / "gate-results/records.tar.gz"
    expected = (archive.parent / "SHA256SUMS").read_text().splitlines()[0].split()[0]
    if sha256(archive.read_bytes()).hexdigest() != expected:
        raise ValueError("archive hash mismatch")
    destination.mkdir(parents=True, exist_ok=False)
    with tarfile.open(archive) as stream:
        stream.extractall(destination, filter="data")
    for line in (destination / "SHA256SUMS").read_text().splitlines():
        digest, name = line.split("  ", 1)
        if sha256((destination / name).read_bytes()).hexdigest() != digest:
            raise ValueError("member hash mismatch: " + name)
    # Original absolute paths are provenance; measures needs paths in this copy.
    for path in sorted(destination.glob("slot-*.json")):
        record = load(path)
        for attempt in record["attempts"]:
            attempt["session"] = str(destination / "sessions" / Path(attempt["session"]).name)
        path.write_text(json.dumps(record, indent=2) + "\n")
    print("Verified archive and members; relocated 120 slot records")


def sample(order):
    slots = sorted(random.Random(SEED).sample(range(len(order)), 12))
    if {order[n][1] for n in slots} != {"rust", "tide"}:
        raise ValueError("sample must cover both arms")
    return slots


def signature(grade):
    """Compare verdicts and exact test identities, excluding timing/path noise."""
    keys = ("code", "stage", "case", "observed_exit", "test_log", "expected_tests",
            "observed_tests", "review_required", "flags")
    return {"structure": grade["structure"], "timed_out": grade.get("timed_out", False),
            "infrastructure": grade.get("infrastructure", False),
            "results": [{key: row[key] for key in keys if key in row}
                        for row in grade["results"]]}


def regrade(records, gate, output):
    head = subprocess.check_output(["git", "-C", str(gate), "rev-parse", "HEAD"], text=True).strip()
    dirty = subprocess.check_output(["git", "-C", str(gate), "status", "--porcelain"], text=True)
    if head != REVISION or dirty:
        raise ValueError("gate worktree must be clean at R")
    state = load(records / "state.json")
    for name in ("driver", "std"):
        path = Path(state["configs"]["tide"][name])
        if sha256(path.read_bytes()).hexdigest() != state["identity"]["cheap"][name]:
            raise ValueError(name + " identity mismatch")
    output.mkdir(parents=True, exist_ok=False)
    results = []
    for slot in sample(state["order"]):
        saved = records / "sessions" / f"{slot:03}-1"
        session = output / saved.name
        shutil.copytree(saved, session)
        row = {"slot": slot, "triple": state["order"][slot]}
        for phase, flags in (("first", ["--first", "--compile-only"]), ("final", [])):
            expected = load(saved / f"{phase}-grade.json")
            with (session / f"regrade-{phase}.log").open("w") as log:
                run = subprocess.run([sys.executable, str(gate / "trials/e1-checked/harness.py"),
                                      "grade", str(session), *flags], cwd=gate,
                                     env=dict(os.environ, TMPDIR="/tmp", CARGO_BUILD_JOBS="4"),
                                     stdin=subprocess.DEVNULL, stdout=log, stderr=subprocess.STDOUT,
                                     timeout=600)
            actual = load(session / f"{phase}-grade.json")
            expected_exit = 0 if (phase == "first" or expected["structure"]) and all(
                r["code"] == 0 for r in expected["results"]) else 1
            row[phase] = run.returncode == expected_exit and signature(actual) == signature(expected)
            if not row[phase]:
                raise ValueError(f"regrade mismatch at slot {slot:03} {phase}; see {session}")
        results.append(row)
        (output / "matches.json").write_text(json.dumps(results, indent=2) + "\n")
        print(f"slot {slot:03}: first and final match", flush=True)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    actions = parser.add_subparsers(dest="action", required=True)
    actions.add_parser("unpack").add_argument("destination", type=Path)
    grade = actions.add_parser("regrade")
    for name in ("records", "gate", "output"):
        grade.add_argument(name, type=Path)
    args = parser.parse_args()
    if args.action == "unpack":
        unpack(args.destination.resolve())
    else:
        regrade(args.records.resolve(), args.gate.resolve(), args.output.resolve())


if __name__ == "__main__":
    main()
