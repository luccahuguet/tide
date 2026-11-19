#!/usr/bin/env python3
"""fx-zu4f.15 pool selection: rows of fx-zu4f.8's committed record in the
same conversion class as chrono_anchor.

Run from the repository root, with no arguments:

    python3 -I research/protocols/fx-zu4f.15-pool/select.py

It reads research/47-malware-rescreen.md at fx-zu4f.8's landing merge
(RECORD_COMMIT) through `git show`, so the working tree does not matter, and
looks up type definitions in Tide's std at fx-zu4f.8's run revision R through
`git grep`. It compiles and runs nothing else, fetches nothing, and prints
no timestamp, so its output is the same on every run.

The rule (fx-zu4f.15's protocol, section 3) selects a row when all hold:

  C1  its capability_run cell records a run of the capability-passing
      version that exited non-zero ("exit N; ..." with N != 0);
  C2  every diagnostic that cell lists is rustc E0277;
  C3  each E0277 is annotated "`?` cannot convert X to Y";
  C4  each such X is a type that Tide's std at R defines
      (`pub struct X` or `pub enum X` under crates/tide-std/src);
  C5  the record's RULING-0039 list of rows "where a table form brings in
      a `?` the source lacked" (section 2) names the row's crate or the
      first word of its incident (amaperf's seven rows share the incident
      "amaperf cluster"), so the failing `?` is a table form's.
"""

import hashlib
import re
import subprocess
import sys

RECORD_COMMIT = "fc43b37a8852092eecf9fe832e5e06d1a26d10e9"
RECORD_PATH = "research/47-malware-rescreen.md"
RECORD_SHA256 = "6c6111b7789d0fedd34a4439c403c103b2f8a57442360afe9083660ffc3b896e"
R = "972aba0ee5917a0a70e59f6b2e7429e6e2378e74"
STD_SRC = "crates/tide-std/src"
BEAD_NAMED = "RUSTSEC-2026-0039"
COLUMNS = [
    "id", "crate", "incident", "stated_effect", "class", "sources_read",
    "code_quoted", "affected_functions", "buggy_codes", "mapped_ops",
    "capability_run", "outcome", "gaps", "reason",
]


def git(*args):
    out = subprocess.run(["git", *args], stdin=subprocess.DEVNULL,
                         capture_output=True, check=False)
    return out.returncode, out.stdout


def fail(msg):
    print("ERROR: " + msg)
    sys.exit(2)


def split_top(text, sep=","):
    """Split on sep outside parentheses."""
    parts, depth, cur = [], 0, ""
    for ch in text:
        if ch == "(":
            depth += 1
        elif ch == ")":
            depth -= 1
        if ch == sep and depth == 0:
            parts.append(cur)
            cur = ""
        else:
            cur += ch
    parts.append(cur)
    return [p.strip() for p in parts if p.strip()]


def std_defines(name):
    code, out = git("grep", "-n", "-E",
                    r"pub (struct|enum) " + re.escape(name) + r"\b",
                    R, "--", STD_SRC)
    if code not in (0, 1):
        fail("git grep at R failed")
    return out.decode().strip().splitlines()


def main():
    code, raw = git("show", RECORD_COMMIT + ":" + RECORD_PATH)
    if code != 0:
        fail("cannot read %s at %s" % (RECORD_PATH, RECORD_COMMIT))
    digest = hashlib.sha256(raw).hexdigest()
    if digest != RECORD_SHA256:
        fail("record SHA-256 %s, expected %s" % (digest, RECORD_SHA256))
    lines = raw.decode("utf-8").split("\n")

    print("fx-zu4f.15 pool selection")
    print("record: %s at %s" % (RECORD_PATH, RECORD_COMMIT))
    print("record SHA-256: %s" % digest)
    print("std types looked up at R = %s, under %s" % (R, STD_SRC))
    print()

    # Section 7 table: header, then one row per advisory.
    header = "| " + " | ".join(COLUMNS) + " |"
    hdr = [i for i, l in enumerate(lines) if l == header]
    if len(hdr) != 1:
        fail("section 7 header not found exactly once")
    rows = []
    for i, l in enumerate(lines):
        if not l.startswith("| [RUSTSEC-"):
            continue
        cells = [c.strip() for c in l.strip()[1:-1].split(" | ")]
        if len(cells) != len(COLUMNS):
            fail("line %d: %d cells, expected %d" % (i + 1, len(cells), len(COLUMNS)))
        row = dict(zip(COLUMNS, cells))
        m = re.match(r"\[(RUSTSEC-\d{4}-\d{4})\]", row["id"])
        if not m:
            fail("line %d: no advisory ID" % (i + 1))
        row["id"] = m.group(1)
        row["line"] = i + 1
        rows.append(row)
    if len(rows) != 87:
        fail("%d rows, expected 87" % len(rows))
    print("section 7 rows parsed: %d (lines %d-%d)" % (len(rows), rows[0]["line"], rows[-1]["line"]))

    # Section 2: RULING-0039's list of rows whose table form adds a `?`.
    start = [i for i, l in enumerate(lines) if "Rows where a table form brings in a `?`" in l]
    if len(start) != 1:
        fail("RULING-0039 row list not found exactly once")
    para, j = [], start[0]
    while j < len(lines) and lines[j].strip() and not (
            j > start[0] and lines[j].lstrip().startswith("- ")):
        para.append(lines[j].strip())
        j += 1
    para_text = " ".join(para)
    table_q_crates = re.findall(r"([A-Za-z0-9_-]+) \(`", para_text)
    print("RULING-0039 table-form `?` rows (lines %d-%d): %s"
          % (start[0] + 1, j, ", ".join(table_q_crates)))

    # Section 8 final-pass table rows, as a second evidence location.
    final_rows = {}
    for i, l in enumerate(lines):
        m = re.match(r"\| (RUSTSEC-\d{4}-\d{4}) capability \|", l)
        if m:
            final_rows[m.group(1)] = i + 1
    print()

    selected, others = [], {}
    for row in rows:
        cell = row["capability_run"]
        m = re.match(r"exit (\d+); (.*)$", cell)
        if not m:
            key = cell if cell else "(empty)"
            others.setdefault(key, []).append(row["id"])
            continue
        verdicts = []
        c1 = m.group(1) != "0"
        verdicts.append("C1 %s (exit %s)" % ("pass" if c1 else "FAIL", m.group(1)))
        items = split_top(m.group(2))
        codes = []
        for it in items:
            im = re.match(r"([A-Z]+\d{4}) x(\d+)(?: \((.*)\))?$", it)
            if not im:
                fail("line %d: cannot parse diagnostic %r" % (row["line"], it))
            codes.append((im.group(1), im.group(3) or ""))
        c2 = all(c == "E0277" for c, _ in codes)
        verdicts.append("C2 %s (codes: %s)" % ("pass" if c2 else "FAIL",
                                                ", ".join(c for c, _ in codes)))
        conv = []
        c3 = c2
        for c, ann in codes:
            if c != "E0277":
                continue
            am = re.search(r"`\?` cannot convert (\w+) to (\w+)", ann)
            if am:
                conv.append((am.group(1), am.group(2)))
            else:
                c3 = False
        c3 = c3 and bool(conv)
        if not c2:
            verdicts.append("C3 not reached")
        else:
            verdicts.append("C3 %s%s" % ("pass" if c3 else "FAIL",
                                         " (" + "; ".join("%s -> %s" % p for p in conv) + ")" if conv else ""))
        c4 = c3
        if c3:
            for x, _ in conv:
                hits = std_defines(x)
                if hits:
                    verdicts.append("C4 pass (%s defined at R: %s)" % (x, "; ".join(h.split(":", 1)[1] for h in hits)))
                else:
                    c4 = False
                    verdicts.append("C4 FAIL (%s not defined in std at R)" % x)
        else:
            verdicts.append("C4 not reached")
        names = [row["crate"], row["incident"].split(" ", 1)[0]]
        hit = [n for n in names if n in table_q_crates]
        c5 = bool(hit)
        verdicts.append("C5 %s (crate %s, incident %s: %s the RULING-0039 list)" % (
            "pass" if c5 else "FAIL", names[0], names[1],
            hit[0] + " in" if c5 else "neither in"))
        ok = c1 and c2 and c3 and c4 and c5
        print("%s %s: %s" % (row["id"], row["crate"], "SELECTED" if ok else "not selected"))
        print("  evidence: %s line %d (section 7 row)%s" % (
            RECORD_PATH, row["line"],
            "; line %d (section 8 final pass)" % final_rows[row["id"]] if row["id"] in final_rows else ""))
        print("  capability_run: %s" % cell)
        print("  outcome: %s" % row["outcome"])
        for v in verdicts:
            print("  " + v)
        if ok:
            selected.append(row)

    print()
    print("rows with no failing capability-passing run recorded (C1 fails):")
    for key in sorted(others):
        print("  %d: %s" % (len(others[key]), key))
    total = sum(len(v) for v in others.values())
    print("  total %d" % total)
    print()

    ids = [r["id"] for r in selected]
    print("selected by the rule: %s" % (", ".join(ids) if ids else "none"))
    print("bead-named row %s selected by the rule: %s" % (BEAD_NAMED, "yes" if BEAD_NAMED in ids else "NO"))
    pool = sorted(set(ids) | {BEAD_NAMED})
    by_id = {r["id"]: r for r in rows}
    print()
    print("POOL (%d), in ascending ID order:" % len(pool))
    for pid in pool:
        r = by_id[pid]
        print("%s\t%s\t%s line %d" % (pid, r["crate"], RECORD_PATH, r["line"]))


if __name__ == "__main__":
    main()
