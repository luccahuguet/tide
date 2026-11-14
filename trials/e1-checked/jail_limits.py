#!/usr/bin/env python3
"""Run one jail in a memory-capped cgroup leaf, then report the leaf's local OOM evidence and peak.

harness.run starts this under `systemd-run --user --scope -p Delegate=yes`. It is trusted
operator code outside the namespace jail: it never reads candidate output.
Usage: jail_limits.py PARENT DIRECTORY -- COMMAND...
PARENT is the harness pid. Reads DIRECTORY/env.json and DIRECTORY/limits.json and writes
DIRECTORY/report.json. The command starts only while the harness lives and after the jail
leaf's effective limits match limits.json.

The delegated scope holds two leaves. The reporter moves itself into reporter/; the cap sits on
jail/, which bwrap enters before it executes. A cap OOM therefore picks victims only among jail
processes, never this reporter, whatever scores the jail sets.
"""
import ctypes
import json
import os
from pathlib import Path
import signal
import subprocess
import sys

CGROUP_ROOT = Path("/sys/fs/cgroup")
SETUP_EXIT = 125
COUNTERS = ["max", "oom", "oom_kill"]
PR_SET_PDEATHSIG = 1


def set_value(path, value):
    path.write_text(value + "\n")


def jail_first(jail):
    # Runs in the forked child before bwrap executes, so no jail code runs outside jail/.
    set_value(jail / "cgroup.procs", "0")
    # The outer bwrap stays at 1000 outside the jail's reach, so the first cap victim takes the
    # whole jail with it (--die-with-parent); the jail may lower its own processes' scores.
    set_value(Path("/proc/self/oom_score_adj"), "1000")


def counters(group):
    # Local counters: max counts hits of memory.max, oom this cgroup's own OOMs, oom_kill victims
    # of any OOM killer (including host or ancestor OOM).
    return {name: int(value) for name, value in (line.split() for line in (group / "memory.events.local").read_text().splitlines())}


def cpu_usage(group):
    return int(dict(line.split() for line in (group / "cpu.stat").read_text().splitlines())["usage_usec"])


def report(directory, value):
    tmp = directory / "report.tmp"
    tmp.write_text(json.dumps(value) + "\n")
    tmp.replace(directory / "report.json")


def main():
    parent, directory, separator, *cmd = sys.argv[1:]
    if separator != "--" or not cmd: raise SystemExit("usage: jail_limits.py PARENT DIRECTORY -- COMMAND...")
    # The harness may die at any time (operator stop, broker or grader kill, crash). Die with it, so
    # bwrap's --die-with-parent ends the jail; the pid check closes the race before the prctl.
    # PDEATHSIG follows the parent *thread*: harness.run must run on its process's main thread.
    if ctypes.CDLL(None, use_errno=True).prctl(PR_SET_PDEATHSIG, signal.SIGKILL, 0, 0, 0):
        raise OSError(ctypes.get_errno(), "cannot install the reporter's parent-death signal")
    directory = Path(directory)
    if os.getppid() != int(parent):
        report(directory, {"setup_error": f"harness {parent} is gone"})
        return SETUP_EXIT
    expected = json.loads((directory / "limits.json").read_text())
    try:
        scope = CGROUP_ROOT / Path(Path("/proc/self/cgroup").read_text().strip().split("::", 1)[1]).relative_to("/")
        (scope / "reporter").mkdir(exist_ok=True)
        (scope / "jail").mkdir(exist_ok=True)
        jail = scope / "jail"
        # A cgroup with processes cannot enable controllers for its children: leave the scope first.
        set_value(scope / "reporter" / "cgroup.procs", "0")
        set_value(scope / "cgroup.subtree_control", "+memory")
        if "memory" not in (jail / "cgroup.controllers").read_text().split():
            raise ValueError("the jail leaf has no memory controller")
        set_value(jail / "memory.max", expected["memory_max"])
        set_value(jail / "memory.swap.max", expected["memory_swap_max"])
        effective = {"memory_max": (jail / "memory.max").read_text().strip(),
                     "memory_swap_max": (jail / "memory.swap.max").read_text().strip()}
        before = counters(jail)
        int((jail / "memory.peak").read_text())
        cpu_usage(jail)
        if effective != expected or any(name not in before for name in COUNTERS):
            raise ValueError(f"jail limits {effective} differ from {expected}")
        # close_fds=False keeps bwrap's status descriptor, which the harness passed through systemd-run.
        proc = subprocess.Popen(cmd, env=json.loads((directory / "env.json").read_text()), close_fds=False,
                                preexec_fn=lambda: jail_first(jail))
    except (OSError, ValueError, IndexError, KeyError, subprocess.SubprocessError) as error:
        report(directory, {"setup_error": str(error)})
        return SETUP_EXIT
    code = proc.wait()
    after = counters(jail)
    report(directory, {"exit": code, **effective, "memory_peak": int((jail / "memory.peak").read_text()),
                       "cpu_usage_usec": cpu_usage(jail), **{name: after[name] - before[name] for name in COUNTERS}})
    return 128 - code if code < 0 else code


if __name__ == "__main__": sys.exit(main())
