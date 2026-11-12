#!/usr/bin/env python3
"""Gate identities, content-only store scan and fail-closed jail canaries."""
import hashlib
import json
import os
from pathlib import Path
import platform
import socket
import subprocess
import sys
import tempfile
import uuid

import harness as h

MODEL = "claude-opus-5-5"
TOOLS = {"Read", "Write", "Edit", "mcp__e1__check", "mcp__e1__test"}


def save(path, value):
    path = Path(path)
    tmp = path.with_suffix(".tmp")
    tmp.write_text(json.dumps(value, indent=2) + "\n")
    tmp.replace(path)


def output(argv):
    return subprocess.check_output(argv, cwd=h.ROOT, stdin=subprocess.DEVNULL, text=True, timeout=30).strip()


def digest(path):
    with Path(path).open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def revision():
    names = ["harness.py", "doctest-runner.sh", "broker.py", "coordinator.py", "gate_checks.py", "standin.py", "writer_guard.py"]
    return {name: digest(h.HERE / name) for name in names}


def canary_key():
    return {"harness": revision(), "bwrap": output(["bwrap", "--version"]),
            "boot_id": Path("/proc/sys/kernel/random/boot_id").read_text().strip()}


def clean():
    paths = ["crates", "trials/e1-checked", "rust-toolchain.toml", "Cargo.lock",
             "design/reference.md", "design/reference-companion.md"]
    if output(["git", "status", "--porcelain", "--untracked-files=all", "--", *paths]):
        raise ValueError("identity requires a clean toolchain, harness and reference tree")


def cheap(configs, cli, model):
    clean()
    return {"head": output(["git", "rev-parse", "HEAD"]), "driver": digest(configs["tide"]["driver"]),
            "std": digest(configs["tide"]["std"]), "cli_path": str(Path(cli).resolve()),
            "cli_sha256": digest(cli), "cli_version": output([cli, "--version"]), "model": model}


def identity(configs, cli, model=MODEL, jobs=1):
    from coordinator import SEED, order, order_hash, writer_argv
    if not cli or jobs < 1: raise ValueError("identity requires a CLI executable and positive jobs")
    record = {"cheap": cheap(configs, cli, model), "crates_tree": output(["git", "rev-parse", "HEAD:crates"]),
              "nightly": h.NIGHTLY, "harness": revision(), "host": canary_key(),
              "kernel": platform.release(), "python": sys.version, "cpus": os.cpu_count(), "jobs": jobs,
              "effort": "default (flag not passed)", "files": {}, "toolchains": {},
              "seed": SEED, "order_hash": order_hash(order()),
              "writer_argv_template": writer_argv(cli, model, Path("SESSION/mcp.json")),
              "canary_records": ["rust-canary.json", "tide-canary.json"],
              "store_scan_records": ["store-before.json", "store-after.json"]}
    for path in [h.ROOT / "Cargo.lock", h.HERE / "rust/Cargo.lock", h.ROOT / "rust-toolchain.toml",
                 h.ROOT / "design/reference.md", h.ROOT / "design/reference-companion.md"]:
        record["files"][str(path.relative_to(h.ROOT))] = digest(path)
    for arm, config in configs.items():
        record["toolchains"][arm] = {"rustc": output([config["sysroot"] + "/bin/rustc", "-Vv"]),
                                    "cargo": output([config["sysroot"] + "/bin/cargo", "-V"])}
    # fx-ic2 freezes this value; this command only measures the current tracked tree.
    measured = h.sha256()
    for name in output(["git", "ls-files", "--", "trials/e1-checked/suites"]).splitlines():
        measured.update(name.encode() + b"\0" + digest(h.ROOT / name).encode() + b"\n")
    record["suites_tree_sha256"] = measured.hexdigest()
    record["suite_freeze"] = "measured only; fx-ic2 must freeze and record before gate"
    return record


def store_scan(store=Path("/nix/store"), tracked=None):
    if tracked is None:
        names = output(["git", "ls-files", "--", "trials/e1-checked", "design"]).splitlines()
        tracked = [h.ROOT / name for name in names]
    sizes = {}
    for path in tracked:
        sizes.setdefault(path.stat().st_size, {}).setdefault(digest(path), []).append(str(path))
    matches, checked, sized = [], 0, 0
    def fail(error):
        raise error
    for directory, _, files in os.walk(store, onerror=fail):
        for name in files:
            path = Path(directory) / name
            if path.is_symlink() or not path.is_file():
                continue
            sized += 1
            options = sizes.get(path.stat().st_size)
            if options:
                checked += 1
                matching = options.get(digest(path))
                if matching:
                    matches.append({"store": str(path), "tracked": matching})
    return {"store": str(store), "exists": store.exists(), "files_sized": sized,
            "files_hashed": checked, "matches": matches, "passed": not matches}


# Open held-out paths without reading a byte. Only planted controls have their bytes checked.
PROBE = r'''
import errno, json, os, socket, sys, tempfile
from pathlib import Path
spec = json.loads(sys.argv[1])
def exists(path):
    try:
        fd = os.open(path, os.O_RDONLY); os.close(fd)
        return True
    except OSError as error:
        if error.errno not in (errno.ENOENT, errno.EACCES, errno.ENOTDIR): raise
        return False
def connect(address):
    try:
        with socket.create_connection(address, timeout=1): return True
    except OSError as error:
        if error.errno not in (errno.ENETUNREACH, errno.ECONNREFUSED, errno.EHOSTUNREACH, errno.ETIMEDOUT): raise
        return False
with socket.socket() as listener:
    listener.bind(('127.0.0.1', 0)); listener.listen()
    local = connect(listener.getsockname())
checks = {name: exists(path) for name, path in spec['paths'].items()}
checks.update({name: connect(address) for name, address in spec['addresses'].items()})
checks['environment'] = 'E1_CANARY_SECRET' in os.environ
try: socket.getaddrinfo('example.com', 80); checks['dns'] = True
except socket.gaierror: checks['dns'] = False
mounts = Path('/proc/self/mountinfo').read_text().splitlines()
tmpfs = any(line.split()[4] == os.environ['TMPDIR'] and line.split(' - ')[1].split()[0] == 'tmpfs' for line in mounts)
with tempfile.TemporaryDirectory(dir=os.environ['TMPDIR']) as fixture:
    Path(fixture, 'fixture-control').write_text('ok')
    tmpfs = tmpfs and Path(fixture).is_relative_to('/tmp')
print(json.dumps({'sentinel': 'E1-CANARY', 'checks': checks,
 'positive': Path('inside').read_text() == spec['token'] and local, 'tmpfs': tmpfs}))
'''


def canary(arm, config):
    h.STATE.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix="canary-", dir=h.STATE) as directory:
        root = Path(directory)
        work = root / "work"
        work.mkdir()
        other = root / "other-session"
        other.mkdir()
        token = uuid.uuid4().hex
        (other / "secret").write_text(token)
        (work / "inside").write_text(token)
        paths = {"repository": h.ROOT / "AGENTS.md", "design": h.ROOT / "design/reference.md",
                 "suite": h.HERE / "suites/common/A2.rs", "reference": h.source("A2", arm),
                 "task": h.HERE / "tasks.md", "other_session": other / "secret",
                 "sessions": other}
        # Host controls prove both host loopback and an address beyond loopback are reachable.
        with socket.socket() as loop, socket.socket() as lan, socket.socket(socket.AF_INET, socket.SOCK_DGRAM) as route:
            route.connect(("192.0.2.1", 9))
            host = route.getsockname()[0]
            loop.bind(("127.0.0.1", 0)); loop.listen(8)
            lan.bind((host, 0)); lan.listen(8)
            spec = {"token": token, "paths": {name: str(path) for name, path in paths.items()},
                    "addresses": {"host_loopback": loop.getsockname(), "network": lan.getsockname()}}
            cmd = ["/usr/bin/python3", "-c", PROBE, json.dumps(spec)]
            records = []
            for shape in ["compile", "test", "binary"]:
                runtime = shape != "compile"
                env = h.environment(config, work)
                # Deliberately plant the variable in the parent, never in the jail environment.
                previous = os.environ.get("E1_CANARY_SECRET")
                os.environ["E1_CANARY_SECRET"] = token
                try:
                    result = h.run(cmd, config, work, timeout=15, env=env,
                                   read_only=["/usr/bin/python3"] if runtime else ())
                    raw = json.loads(result["stdout"])
                    expected = set(paths) | set(spec["addresses"]) | {"environment", "dns"}
                    if (result["code"] or token in result["stdout"] or raw["sentinel"] != "E1-CANARY"
                        or not raw["positive"] or not raw["tmpfs"] or set(raw["checks"]) != expected
                        or any(raw["checks"].values())):
                        raise ValueError(f"{arm}/{shape} isolation canary failed: {result}")
                    outside_env = dict(os.environ, **h.environment(dict(config, sandbox=False), work))
                    control = h.run(cmd, dict(config, sandbox=False), work, timeout=15, env=outside_env)
                    outside = json.loads(control["stdout"])
                    if (control["code"] or token in control["stdout"] or not outside["positive"]
                        or set(outside["checks"]) != expected or not all(outside["checks"].values())):
                        raise ValueError(f"{arm}/{shape} negative control failed: {control}")
                finally:
                    if previous is None: os.environ.pop("E1_CANARY_SECRET", None)
                    else: os.environ["E1_CANARY_SECRET"] = previous
                records.append({"shape": shape, **raw, "negative_control": outside})
    return {"arm": arm, "key": canary_key(), "passed": True, "records": records}
