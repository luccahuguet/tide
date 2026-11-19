#!/usr/bin/env python3
"""fx-zu4f.13 pool selection: the bead's mechanical capability-only rule.

Reads only committed records (no network, no git, no subprocess) and writes:
  pool.tsv          rows whose ONLY first-failure reason is a capability Tide
                    v0 lacked at the source round's registered revision and
                    which has since landed on main (counted pool)
  udp-only.tsv      rows whose only remaining blocker is UDP (listed, never
                    counted; relayed owner decision, 2026-11-19)
  resolve-only.tsv  rows whose only remaining blocker is Net::resolve (these
                    are also pool rows)
  screen.tsv        every row that passed the gate, with the decision and the
                    matched terms, so each exclusion can be audited
  selection.json    input SHA-256s, the landed-capability table, counts

Usage: python3 -I select.py --root <repo> [--ten-root <dir>] --out <dir>
  --root      a checkout holding the committed records of fx-zu4f.5-.9, .11, .8
  --ten-root  where fx-zu4f.10's record lives with the same relative paths
              (default --root; before .10 lands, an export of branch
              fx-zu4f.10, and every .10 row is then marked provisional)

The rule, as the bead states it:
  1. Gate: a row counts only if its recorded FIRST failure is the round's
     expressibility criterion (E4 in the bug rounds; "not caught" after a
     ported or assessed capability-passing version in fx-zu4f.5;
     "capability absent" in fx-zu4f.8; "unportable" for an arm-A target).
     A row that failed earlier (E1-E3, no portable source, port incomplete,
     no TD code, unresolved) is never in the pool.
  2. Capability-only: in records with a capability column (fx-zu4f.10's
     `cap`, fx-zu4f.11's `cap_tag`) the round's own tag decides. In records
     without one (fx-zu4f.5-.9, .8), the recorded reason must name a
     subprocess or network operation and none of the other grounds below.
     Tagged records are also run through the term test as a cross-check;
     a disagreement is reported, never silently resolved.
  3. Landed: the operation's capability must be absent at the source round's
     registered revision and landed on main now. A network row is matched by
     its operation: UDP -> not landed (UDP-only list); system DNS lookup ->
     Net::resolve; HTTP GET -> Net::get. A network row whose operation the
     record does not name is reported as unclassified for a manual reading.
"""
import argparse
import csv
import hashlib
import json
import re
import sys
from pathlib import Path

# Capabilities landed on main since fx-zu4f.5 (commit ancestry is checked at
# registration with `git merge-base --is-ancestor <commit> R`).
LANDED = {
    "proc": {"capability": "Proc::command", "bead": "fx-ayh2.1",
             "commit": "e0b0d6216bed121c784c76e27b10582a2e0053ed"},
    "get": {"capability": "Net::get", "bead": "fx-ff13",
            "commit": "8ac13175c98ec29b3f313a860d657e1c34c1fafe"},
    "resolve": {"capability": "Net::resolve", "bead": "fx-6d50",
                "commit": "d378a7d39e0edc70cbb409e53d28ad042f0c1606"},
}
NOT_LANDED = {"udp": "UDP (bind_udp/UdpSocket, UDP DNS): planned, no bead"}

# Source rounds: record files, registered revision, capabilities already in
# v0 at that revision (from `git merge-base --is-ancestor`, 2026-11-19).
SOURCES = {
    "fx-zu4f.5": {"record": "research/37-external-admission-search.md",
                  "revision": "5dc6b40e0254d5dd83ab83b0f2d5a1dd2c2ec6dc",
                  "present": []},
    "fx-zu4f.6": {"record": "research/38-external-concurrency-search.md",
                  "revision": "7863e8f54110a63e5e435999a0a52182d433704e",
                  "present": []},
    "fx-zu4f.7": {"record": "research/39-std-concurrency-search.md",
                  "revision": "afb17ea7af79c1db2d6a55322f2ab56a209ce845",
                  "present": []},
    "fx-zu4f.8": {"record": "research/47-malware-rescreen.md",
                  "revision": "972aba0ee5917a0a70e59f6b2e7429e6e2378e74",
                  "present": ["proc", "get"]},
    "fx-zu4f.9": {"record": "research/41-qin-dataset-search.md",
                  "revision": "1e673c525ca7d0a24c881d664a2bf98d78717cef",
                  "present": []},
    "fx-zu4f.10": {"record": "research/44-fx-zu4f.10-concurrency-round.md",
                   "ledger": "research/protocols/fx-zu4f.10-record/arm-b-ledger.tsv",
                   "revision": "a28b4df0b49731a20e9f181ab788fc5ce498a53a",
                   "present": []},
    "fx-zu4f.11": {"record": "research/46-fx-zu4f.11-agent-marker-round.md",
                   "ledger": "research/protocols/fx-zu4f.11-record/b-candidates.tsv",
                   "revision": "ea7bb313eac1a66f37b643393bbd5f7dfdeb91e2",
                   "present": []},
}

# Capability terms (case-sensitive on purpose: "Socket" is a vendor name in
# the malware records, "network/" a source directory in fx-zu4f.9).
CAP_TERMS = {
    "proc": re.compile(r"\bsubprocess(es)?\b|\bchild process|\b(clang|PowerShell|rustfmt) child"
                       r"|`Proc`|\bProc::|`Command::|\bCommand::|\bcap:proc\b"),
    "net": re.compile(r"\bnetwork\b(?!/)|\bHTTP\b|\bTCP\b|\bsocket\b|\bUDP\b|UdpSocket"
                      r"|\bDNS\b|\bNet::|\bcap:net\b"),
}
# Every other first-failure ground the bead names (async, external crates, a
# core over 400 lines, an OS trigger, memory unsafety, unresolved dispatch),
# plus the grounds the source rounds recorded.
OTHER_TERMS = {
    "async": r"\basync\b|\bawait\b|\bFuture\b|\bWaker\b|\btokio\b",
    "external crate": r"(?i:\bexternal\b|\bthird-party\b)|\bE0433\b|\b(reqwest|hyper|serde_json|cfg_if|mio"
                      r"|parking_lot|ipc[-_]channel|rustls|timely|pyo3|rusqlite|egui|fs4|glfw"
                      r"|glutin|ctor|lock_api|crossbeam|sysinfo|cpal)\b",
    "OS or kernel trigger": r"\bkernel\b|\bOS\b|\bsetsockopt\b|\bEINVAL\b|\bRST\b|\bIOCP\b|io_uring"
                            r"|named pipe|Unix(-| domain )?socket|\bpermissions?\b|OS-error",
    "memory unsafety": r"\bunsafe\b|memory[- ]unsafety|\bMiri\b|\bMIRI\b|\bTSan\b|ThreadSanitizer"
                       r"|\bsegfault\b|raw[- ]pointer|weak[- ]memory|data race",
    "signal or process exit": r"\bsignals?\b|\bSIGCHLD\b|process-exit|exit ordering",
    "lock shape": r"\bMutex\b|\bRwLock\b|\bCondvar\b|\bpoison|guard held|held across|re-?enter"
                  r"|nested lock|same-lock|lock held",
    "not Rust": r"\bPython\b",
    "size": r"over 400|400-line|400 lines",
    "unresolved dispatch": r"unresolved dispatch",
    "FFI": r"\bFFI\b",
    "unmapped operation": r"no (registered )?mapping|no table row|\(no row\)|has no mapping",
}
OTHER_RE = {k: re.compile(v) for k, v in OTHER_TERMS.items()}

# Network operation families, matched in this order.
NET_OPS = [
    ("udp", re.compile(r"\bUDP\b|UdpSocket|bind_udp")),
    ("resolve", re.compile(r"\bresolve\b|system resolver|\bnslookup\b|getaddrinfo"
                           r"|lookup_host|ToSocketAddrs")),
    ("get", re.compile(r"\bHTTP\b|\bGET\b")),
]


def sha256(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def classify(text):
    caps = sorted(k for k, rx in CAP_TERMS.items() if rx.search(text))
    other = sorted(k for k, rx in OTHER_RE.items() if rx.search(text))
    return caps, other


def net_op(text):
    for name, rx in NET_OPS:
        if rx.search(text):
            return name
    return "unclassified"


def row(src, rid, first, gate, reason, tag="", evidence="", tagged_record=False):
    return {"source": src, "row": rid, "first_failure": first, "gate": gate,
            "reason": " ".join(reason.split()), "tag": tag,
            "evidence": " ".join(evidence.split()), "tagged_record": tagged_record}


def cells(line):
    """Split a markdown table row on ' | ' outside backticks."""
    out, cur, tick = [], [], False
    body = line.strip()[1:-1] if line.strip().endswith("|") else line.strip()[1:]
    for ch in body:
        if ch == "`":
            tick = not tick
        if ch == "|" and not tick:
            out.append("".join(cur).strip())
            cur = []
        else:
            cur.append(ch)
    out.append("".join(cur).strip())
    return out


# ---- per-record extractors -------------------------------------------------

def rows_r5(text):
    for line in text.splitlines():
        if not line.startswith("| [RUSTSEC-"):
            continue
        c = cells(line)
        rid = re.match(r"\[(RUSTSEC-\d{4}-\d{4})\]", c[0]).group(1) + " " + c[1].strip("`")
        outcome, reason = c[5], c[6]
        yield row("fx-zu4f.5", rid, outcome, outcome == "not caught", reason)


def rows_r6(text):
    for m in re.finditer(r"^- \*\*(A\d+)\*\* (.*?)(?=^- \*\*A\d+\*\*|^## )", text, re.S | re.M):
        rid, body = m.group(1), m.group(2)
        f = re.search(r"\*\*(E\d) ✗\*\*:?(.*)", body)
        first = f.group(1) if f else "none recorded"
        yield row("fx-zu4f.6", rid + " " + body.split("\n")[0][:80], first,
                  first == "E4", f.group(2) if f else "")


def rows_r7(text):
    for line in text.splitlines():
        m = re.match(r"^\| (R\d+T\d+\.\d+) \|", line)
        if not m:
            continue
        c = cells(line)
        outcome = c[3]
        e = re.search(r"\((E\d)\)", outcome)
        first = e.group(1) if e else outcome
        yield row("fx-zu4f.7", m.group(1) + " " + c[1], first, first == "E4", c[4])


def rows_r9(text):
    for line in text.splitlines():
        m = re.match(r"^\| ([BN]\d{2}) \| ", line)
        if not m:
            continue
        c = cells(line)
        if len(c) < 6 or not re.fullmatch(r"E\d", c[4]):
            continue
        yield row("fx-zu4f.9", f"{c[0]} {c[1].strip('`')} {c[2]}", c[4], c[4] == "E4", c[5])


def rows_r8(text):
    for line in text.splitlines():
        if not line.startswith("| [RUSTSEC-"):
            continue
        rid = re.match(r"\| \[(RUSTSEC-\d{4}-\d{4})\]", line).group(1)
        m = re.search(r"not caught \(([^)]+)\)", line)
        outcome = m.group(1) if m else "unparsed"
        tail = line[m.end():] if m else line
        yield row("fx-zu4f.8", rid, outcome, outcome == "capability absent", tail)


def cap_table(text, heading):
    """Rows of a record's 'Capability-only failures' table, as raw lines."""
    i = text.find(heading)
    if i < 0:
        return []
    out = []
    for line in text[i:].splitlines()[1:]:
        if line.startswith("## ") or line.startswith("### "):
            break
        if line.startswith("| ") and "---" not in line:
            out.append(line)
    return out


def rows_r10(record, ledger):
    caps = cap_table(record, "### Capability-only failures")
    with ledger.open(newline="") as fh:
        for r in csv.DictReader(fh, delimiter="\t"):
            first = r["first_failure"]
            ev = " ".join(l for l in caps if r["id"] in l)
            tag = r["cap"].strip()
            tt = re.search(r"\b(cap:\w+)\b", ev)
            if tt and not tag.startswith("cap:"):
                # The record's capability-only table tags a row whose ledger cell
                # is blank. The table is part of the committed record, so its tag
                # counts; the mismatch stays visible in the tag text.
                tag = tt.group(1) + " (record table; ledger cap empty)"
            yield row("fx-zu4f.10", f"arm B {r['id']} {r['crate']} {r['commit'][:12]}", first,
                      first == "E4", r["reason"], tag, ev, True)
    # Arm A: the pool table's arm-A column (the first table after its heading).
    i = record.find("All 24 pool crates")
    j = record.find("\n## ", i)
    for line in record[i:j].splitlines():
        m = re.match(r"^\| (\d+) \| (.+?) \| .+ \| (unportable: .*|no concurrent core|not targeted"
                     r"|portable.*|modeled.*) \|$", line)
        if not m:
            continue
        verdict = m.group(3)
        tag = re.search(r"\((cap:\w+)\)", verdict)
        ev = " ".join(l for l in caps if f"rank {m.group(1)}," in l)
        yield row("fx-zu4f.10", f"arm A rank {m.group(1)} {m.group(2)}",
                  verdict.split(":")[0], verdict.startswith("unportable"), verdict,
                  tag.group(1) if tag else "", ev, True)


def rows_r11(record, ledger):
    caps = cap_table(record, "## Capability-only failures")
    with ledger.open(newline="") as fh:
        for r in csv.DictReader(fh, delimiter="\t"):
            first = r["first_failure"]
            ev = " ".join(l for l in caps if r["commit"][:12] in l)
            yield row("fx-zu4f.11", f"arm B rank {r['rank']} {r['repository']} {r['commit'][:12]}",
                      first, first == "E4", r["reason"], r["cap_tag"].strip(), ev, True)
    i = record.find("## Arm A")
    for line in record[i:].splitlines():
        m = re.match(r"^\| (\d) \| (.*) \| (.*) \|$", line)
        if m:
            yield row("fx-zu4f.11", f"arm A rank {m.group(1)}", "unportable", True,
                      m.group(3), "", "", True)
        if line.startswith("No arm-A repository"):
            break


# ---- decision ---------------------------------------------------------------

def decide(r):
    src = SOURCES[r["source"]]
    text = r["reason"] + " " + r["evidence"]
    caps, other = classify(r["reason"])
    r["cap_terms"], r["other_grounds"] = ",".join(caps), ",".join(other)
    if r["tagged_record"]:
        fam = r["tag"].split()[0].removeprefix("cap:") if r["tag"].startswith("cap:") else ""
        term_says = bool(caps) and not other
        if bool(fam) != term_says:
            r["crosscheck"] = ("tagged, but the term test finds other grounds: " + r["other_grounds"]
                               if fam else "untagged, but the term test finds capability-only")
        else:
            r["crosscheck"] = "agrees"
        if not fam:
            return "not capability-only (round's tag)"
    else:
        r["crosscheck"] = ""
        if not caps:
            return "no capability operation named"
        if other:
            return "other grounds: " + r["other_grounds"]
        fam = "+".join(caps)  # proc, net, or proc+net (each must be landed)
    ops = []
    for f in fam.split("+"):
        ops.append("proc" if f == "proc" else net_op(text))
    r["operation"] = ",".join(ops)
    if any(o == "unclassified" for o in ops):
        return "network operation unclassified (manual reading)"
    if any(o in NOT_LANDED for o in ops):
        return "UDP-only" if all(o in NOT_LANDED or (o in LANDED and o not in src["present"])
                                 for o in ops) else "excluded"
    if any(o in src["present"] for o in ops):
        return "capability already in v0 at the source revision"
    return "POOL"


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--root", required=True, type=Path)
    ap.add_argument("--ten-root", type=Path)
    ap.add_argument("--out", required=True, type=Path)
    a = ap.parse_args()
    ten = a.ten_root or a.root
    provisional_ten = a.ten_root is not None and a.ten_root.resolve() != a.root.resolve()
    inputs, rows = {}, []

    def read(base, rel):
        p = base / rel
        inputs[str(p)] = sha256(p)
        return p.read_text(encoding="utf-8")

    rows += rows_r5(read(a.root, SOURCES["fx-zu4f.5"]["record"]))
    rows += rows_r6(read(a.root, SOURCES["fx-zu4f.6"]["record"]))
    rows += rows_r7(read(a.root, SOURCES["fx-zu4f.7"]["record"]))
    rows += rows_r8(read(a.root, SOURCES["fx-zu4f.8"]["record"]))
    rows += rows_r9(read(a.root, SOURCES["fx-zu4f.9"]["record"]))
    rec10 = read(ten, SOURCES["fx-zu4f.10"]["record"])
    read(ten, SOURCES["fx-zu4f.10"]["ledger"])
    rows += rows_r10(rec10, ten / SOURCES["fx-zu4f.10"]["ledger"])
    rec11 = read(a.root, SOURCES["fx-zu4f.11"]["record"])
    read(a.root, SOURCES["fx-zu4f.11"]["ledger"])
    rows += rows_r11(rec11, a.root / SOURCES["fx-zu4f.11"]["ledger"])

    counts = {}
    for r in rows:
        r["provisional"] = "yes" if (r["source"] == "fx-zu4f.10" and provisional_ten) else ""
        c = counts.setdefault(r["source"], {"rows": 0, "gate": 0, "capability_named": 0,
                                            "pool": 0, "udp_only": 0, "resolve_only": 0,
                                            "manual": 0, "crosscheck_disagreements": 0})
        c["rows"] += 1
        r.setdefault("operation", "")
        r.setdefault("cap_terms", "")
        r.setdefault("other_grounds", "")
        r.setdefault("crosscheck", "")
        if not r["gate"]:
            r["decision"] = "gate: first failure " + r["first_failure"]
            continue
        c["gate"] += 1
        r["decision"] = decide(r)
        if r["cap_terms"] or r["tag"]:
            c["capability_named"] += 1
        if r["decision"] == "POOL":
            c["pool"] += 1
            if "resolve" in r["operation"].split(","):
                c["resolve_only"] += 1
        elif r["decision"] == "UDP-only":
            c["udp_only"] += 1
        elif "manual" in r["decision"]:
            c["manual"] += 1
        if r["crosscheck"] not in ("", "agrees"):
            c["crosscheck_disagreements"] += 1

    a.out.mkdir(parents=True, exist_ok=True)
    cols = ["source", "row", "first_failure", "tag", "cap_terms", "other_grounds", "operation",
            "decision", "crosscheck", "provisional", "reason", "evidence"]

    def write(name, sel):
        with (a.out / name).open("w", newline="") as fh:
            w = csv.writer(fh, delimiter="\t", lineterminator="\n")
            w.writerow(cols + ["capability", "bead", "landing_commit"])
            for r in sel:
                ops = [o for o in r["operation"].split(",") if o]
                land = [LANDED.get(o, {"capability": NOT_LANDED.get(o, o), "bead": "", "commit": ""})
                        for o in ops]
                w.writerow([r[k] for k in cols] + [";".join(x["capability"] for x in land),
                                                   ";".join(x["bead"] for x in land),
                                                   ";".join(x["commit"] for x in land)])

    gated = [r for r in rows if r["gate"]]
    write("screen.tsv", [r for r in gated if r["cap_terms"] or r["tag"] or
                         r["crosscheck"] not in ("", "agrees")])
    write("pool.tsv", [r for r in gated if r["decision"] == "POOL"])
    write("udp-only.tsv", [r for r in gated if r["decision"] == "UDP-only"])
    write("resolve-only.tsv", [r for r in gated if r["decision"] == "POOL"
                               and "resolve" in r["operation"].split(",")])
    summary = {"inputs_sha256": inputs, "fx-zu4f.10_provisional": provisional_ten,
               "landed": LANDED, "not_landed": NOT_LANDED,
               "sources": {k: {"revision": v["revision"], "present_at_revision": v["present"]}
                           for k, v in SOURCES.items()},
               "counts": counts}
    (a.out / "selection.json").write_text(json.dumps(summary, indent=2) + "\n")
    for k, c in counts.items():
        print(k, json.dumps(c))
    return 0


if __name__ == "__main__":
    sys.exit(main())
