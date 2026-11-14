#!/usr/bin/env python3
"""Gate identities, content-only store scan and fail-closed jail canaries."""
import errno
import hashlib
import json
import os
from pathlib import Path
import platform
import socket
import stat
import subprocess
import sys
import tempfile
import uuid

import harness as h

MODEL = "claude-opus-5-5"
TOOLS = {"Read", "Write", "Edit", "mcp__e1__check", "mcp__e1__test"}
# Managed Claude Code configuration. Without --safe-mode the CLI reads managed output styles here
# even under --restricted, and a style named "default" replaces the system prompt while init
# still passes, so its presence in any form refuses every real writer launch.
MANAGED_CONFIG = Path("/etc/claude-code")


def managed_config_absent():
    try: os.lstat(MANAGED_CONFIG)
    except FileNotFoundError: return {"path": str(MANAGED_CONFIG), "exists": False}
    raise ValueError(f"{MANAGED_CONFIG} exists (directory, file or symlink): managed Claude Code configuration "
                     "can change the writer's instructions unseen; remove it before any E1 writer launch")


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
    names = ["harness.py", "doctest-runner.sh", "broker.py", "coordinator.py", "gate_checks.py", "standin.py", "writer_guard.py", "jail_limits.py"]
    return {name: digest(h.HERE / name) for name in names}


def canary_key():
    return {"harness": revision(), "bwrap": output([h.system_tool("bwrap"), "--version"]),
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
              "jail_limits": h.limits(), "store_closure": {arm: config["store_paths"] for arm, config in configs.items()},
              "effort": "default (flag not passed)", "files": {}, "toolchains": {},
              "seed": SEED, "order_hash": order_hash(order()),
              "writer_argv_template": writer_argv(cli, model, Path("SESSION/mcp.json")),
              "managed_config": managed_config_absent(),
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


def writer_visible(path, trial=h.HERE):
    """Files every writer receives or that carry no trial content; byte matches on them prove nothing."""
    path = Path(path)
    return (path.name == ".gitignore" or path.stat().st_size == 0
            or path in {trial / "rust" / name for name in h.RUST_WRITER_FILES}
            or path in {h.ROOT / "design" / name for name in h.TIDE_WRITER_DOCS})


def store_scan(store=h.STORE, tracked=None, trial=h.HERE):
    """Flag store files byte-identical to held-out trial/design files, and any store path holding
    trials/e1-checked content at any revision: an e1-checked directory, or a path ending in a
    tracked held-out path (suites/... or references/...). Writer-visible files are not byte-matched
    (unrelated trees hold copies of them). A store file that cannot be opened, or a directory that
    cannot be searched, is recorded and skipped only when the denial is exactly EACCES and another uid
    owns it (such as a root-owned 0700 Nix build chroot): that owner is unmapped in a jail, so no
    nested user namespace regains access. Content this uid owns but cannot read is fatal, since a jail
    can regain owner capabilities over it; so are a searchable directory that cannot be listed and
    every other error."""
    if tracked is None:
        names = output(["git", "ls-files", "--", "trials/e1-checked", "design"]).splitlines()
        tracked = [h.ROOT / name for name in names]
    sizes, suffixes = {}, set()
    for path in tracked:
        if not writer_visible(path, trial):
            sizes.setdefault(path.stat().st_size, {}).setdefault(digest(path), []).append(str(path))
        if path.is_relative_to(trial) and path.relative_to(trial).parts[0] in {"suites", "references"}:
            suffixes.add(path.relative_to(trial).parts)
    lengths = {len(parts) for parts in suffixes}
    matches, checked, sized, holders, unreadable = [], 0, 0, {}, []
    def foreign_denial(error, owner):
        return error.errno == errno.EACCES and owner != os.getuid()
    def search_denied(directory):
        # An errno-preserving search probe (os.access would turn EIO into "denied").
        try: os.close(os.open(os.path.join(directory, "."), os.O_PATH | os.O_DIRECTORY))
        except OSError as error:
            if foreign_denial(error, os.lstat(directory).st_uid): return True
            raise
        return False
    def listing_failed(error):
        if isinstance(error, PermissionError) and search_denied(error.filename):
            unreadable.append(error.filename); return
        raise error
    for directory, dirs, files in os.walk(store, onerror=listing_failed):
        relative = Path(directory).relative_to(store).parts
        # Compare names without the store hash, so a copy at another revision still matches.
        named = relative[:1] and (relative[0].partition("-")[2] or relative[0],) + relative[1:]
        for name in dirs + files:
            parts = named + (name,) if named else ((name.partition("-")[2] or name),)
            if parts[-1] == "e1-checked" or any(parts[-n:] in suffixes for n in lengths):
                entry = str(store / (relative[0] if relative else name))
                holders.setdefault(entry, "/".join(parts))
        if search_denied(directory):  # Listable names, but nothing inside can be opened.
            unreadable.append(directory); dirs[:] = []; continue
        for name in files:
            path = Path(directory) / name
            # Explicit lstat: Path.is_file() would swallow EIO and silently skip the file.
            info = os.lstat(path)
            if stat.S_ISLNK(info.st_mode): continue  # Store symlinks resolve to paths visited directly.
            if stat.S_ISDIR(info.st_mode): raise OSError(f"store directory was not walked: {path}")
            if not stat.S_ISREG(info.st_mode): continue
            sized += 1
            options = sizes.get(info.st_size)
            if options:
                checked += 1
                try: matching = options.get(digest(path))
                except PermissionError as error:
                    if not foreign_denial(error, info.st_uid): raise
                    unreadable.append(str(path)); continue
                if matching:
                    matches.append({"store": str(path), "tracked": matching})
    matches += [{"store": entry, "trial_path": path} for entry, path in sorted(holders.items())]
    return {"store": str(store), "exists": store.exists(), "files_sized": sized,
            "files_hashed": checked, "unreadable_skipped": sorted(unreadable), "matches": matches, "passed": not matches}


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
size = os.statvfs('/tmp')
print(json.dumps({'sentinel': 'E1-CANARY', 'checks': checks,
 'positive': Path('inside').read_text() == spec['token'] and local, 'tmpfs': tmpfs,
 'tmpfs_bytes': size.f_blocks * size.f_frsize}))
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
        # A store entry outside the toolchain closure must be invisible, as any older trial copy is.
        closure = set(config.get("store_paths", []))
        outside = next((h.STORE / name for name in sorted(os.listdir(h.STORE)) if not name.startswith(".")
                        and str(h.STORE / name) not in closure), None) if h.STORE.is_dir() else None
        if outside is not None: paths["store_outside_closure"] = outside
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
                        or not raw["positive"] or not raw["tmpfs"] or raw.get("tmpfs_bytes") != h.TMPFS_SIZE
                        or set(raw["checks"]) != expected
                        or any(raw["checks"].values())):
                        raise ValueError(f"{arm}/{shape} isolation canary failed: {result}")
                    outside_env = dict(os.environ, **h.environment(dict(config, sandbox=False), work))
                    control = h.run(cmd, dict(config, sandbox=False), work, timeout=15, env=outside_env)
                    control_raw = json.loads(control["stdout"])
                    if (control["code"] or token in control["stdout"] or not control_raw["positive"]
                        or set(control_raw["checks"]) != expected or not all(control_raw["checks"].values())):
                        raise ValueError(f"{arm}/{shape} negative control failed: {control}")
                finally:
                    if previous is None: os.environ.pop("E1_CANARY_SECRET", None)
                    else: os.environ["E1_CANARY_SECRET"] = previous
                records.append({"shape": shape, **raw, "negative_control": control_raw})
    return {"arm": arm, "key": canary_key(), "passed": True, "records": records}
