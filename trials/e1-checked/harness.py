#!/usr/bin/env python3
"""E1 preparation and readiness checks. This is not a gate-session coordinator."""
import argparse
import fcntl
from hashlib import sha256
import json
import os
from pathlib import Path
import re
import shutil
import signal
import subprocess
import sys
import tempfile
import time
import tomllib

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]
STATE = ROOT / "target/e1-checked"
RUST = "1.96.0"
NIGHTLY = tomllib.loads((ROOT / "rust-toolchain.toml").read_text())["toolchain"]["channel"]
BINS = {"B1", "B2", "B5", "B8"}
TASKS = [f"{band}{i}" for band in "ABC" for i in range(1, 11)]
# Freeze with the suites; candidate output never defines acceptance coverage.
ACCEPTANCE_TESTS = {
    "A1": "case1 case2 case3 case4 case5",
    "A2": "case1 case2 case3 case4 case5 case6 case6_zero",
    "A3": "case1 case2 case3 case4 case5",
    "A4": "case1 case2 case3 case4 case5",
    "A5": "case1 case2 case3 case4 case5",
    "A6": "case1 case2 case3 case4 case5",
    "A7": "case1 case2 case3 case4 case5",
    "A8": "case1 case2 case3 case4 case5",
    "A9": "case1 case2 case3 case4 case5",
    "A10": "case1 case2 case3 case4",
    "B3": "case1 case2 case3 case4",
    "B4": "case1 case2 case3 case4 case5",
    "B6": "case1 case2 case3 case4 case5",
    "B7": "case1 case2 case3 case4 case5",
    "B9": "case1 case2 case3 case4 case5",
    "B10": "case1 case2 case3 case4",
    "C1": "case1 case2 case3 case3_equal_edges case3_short_edges",
    "C2": "case1 case2 case3",
    "C3": "case1 case2 case3 case4",
    "C4": "case1 case2 case3 case4",
    "C5": "case1 case2 case3",
    "C6": "case1 case2 case3 case4 case5",
    "C7": "case1 case2_drain case2_new case2_push case3 case4",
    "C8": "case1 case2 case3 case4",
    "C9": "case1 case2_overflow case2_zero case3",
    "C10": "case1 case2",
}


def task_text(task, arm):
    text = (HERE / "tasks.md").read_text()
    preface = text.split("## Preface\n", 1)[1].split("## Band A:", 1)[0]
    match = re.search(rf"^### {task}\. .+?(?=^### |^## Band |\Z)", text, re.M | re.S)
    if match is None:
        raise ValueError(f"unknown task: {task}")
    lines = []
    keep = True
    for line in (preface + match[0]).splitlines(keepends=True):
        label = re.match(r"- (Tide|Rust): (.*)", line)
        if label:
            keep = label[1].lower() == arm
            if keep:
                lines.append("- " + label[2] + ("\n" if line.endswith("\n") else ""))
        elif line.startswith("  "):
            if keep:
                lines.append(line)
        else:
            keep = True
            lines.append(line)
    return "".join(lines)


def source(task, arm):
    path = HERE / f"references/{arm}/{task}.rs"
    return path if path.exists() else HERE / f"references/common/{task}.rs"


def structure(task, arm, text):
    """Literal searches mandated by tasks.md; comments deliberately count."""
    checks = []
    if arm == "tide":
        if task in BINS:
            checks.append("#[tide::main]" in text)
        if task == "B6": checks.append(".dir(" in text)
        if task == "B7": checks.append(re.search(r"#\[pure\]\s*pub\s+fn\s+plan\b", text) is not None)
        if task == "B9": checks.extend(token in text for token in ["write_synced(", "sync_dir("])
        if task == "C1": checks.append(any(t in text for t in ["par_map(", "par_for(", "std::par"]))
        if task == "C2": checks.append("std::par" in text and "par_sort" not in text)
        if task == "C4": checks.append("Shared::new(" in text)
        if task == "C6": checks.append("checkpoint(" in text or ".cancel(" in text)
        if task == "C7": checks.extend(text.count(t) >= 2 for t in ["#[requires(", "#[ensures("])
        if task == "C8": checks.extend(t in text for t in ["#[ensures(", "invariant!(", "#[requires("])
    else:
        if task == "B9": checks.append("sync_all(" in text)
        if task in {"C1", "C2"}: checks.append("thread::scope(" in text)
        if task == "C4": checks.append("Mutex::new(" in text)
        if task == "C6": checks.append("AtomicBool" in text)
    if task == "C3": checks.append("sync_channel(" in text)
    return all(checks)


def suite(task, arm):
    path = HERE / f"suites/{arm}/{task}.rs"
    if not path.exists(): path = HERE / f"suites/common/{task}.rs"
    text = path.read_text()
    if task in {"B3", "B4", "B6", "B7", "B9"}:
        text = (HERE / f"suites/{arm}/fs.rs").read_text() + text
    if task == "B3":
        if arm == "rust":
            text += '\nfn summarize_at(fs:&Fixture,p:&str)->std::io::Result<Summary> { solution::summarize(&fs.join(p)) }\nfn top_at(fs:&Fixture,p:&str,n:usize)->std::io::Result<Vec<(String,u64)>> { solution::top_n(&fs.join(p),n) }\n'
        else:
            text += '\nfn summarize_at(fs:&Fixture,p:&str)->std::io::Result<Summary> { solution::summarize(fs,p) }\nfn top_at(fs:&Fixture,p:&str,n:usize)->std::io::Result<Vec<(String,u64)>> { solution::top_n(fs,p,n) }\n'
    if task in {"C1", "C2", "C8"}:
        text += (HERE / "suites/common/random.rs").read_text()
    return "use solution::*;\n" + text


def acceptance_tests(task):
    return ACCEPTANCE_TESTS[task].split()


def integrity_flags(text):
    # Literal searches require review, not a claim to detect every in-process forgery.
    patterns = [("process termination", r"\bprocess\s*::\s*(?:exit|abort)\b"),
                ("unsafe or pre-main linkage", r"#\s*\[\s*unsafe\s*\(|\b(?:link_section|export_name|no_mangle|ctor)\b"),
                ("libtest output", r"\btest\s+result\b")]
    return [reason for reason, pattern in patterns if re.search(pattern, text)]


def toolchain(name):
    # Resolve once outside every candidate. Never use a candidate's rustup configuration.
    return Path(subprocess.check_output(["rustup", "run", name, "rustc", "--print", "sysroot"], text=True).strip())


def rust_vendor():
    return offline_vendor("rust", HERE / "rust", RUST)


def tide_vendor():
    return offline_vendor("tide", ROOT, NIGHTLY)


def offline_vendor(arm, project, channel):
    lock = project / "Cargo.lock"
    pinned = lock.read_bytes()
    STATE.mkdir(parents=True, exist_ok=True)
    dest = STATE / f"{arm}-vendor-{sha256(pinned).hexdigest()[:16]}"
    if dest.exists():
        return dest
    tmp = Path(tempfile.mkdtemp(prefix=f"{arm}-vendor-", dir=STATE))
    try:
        result = subprocess.run([str(toolchain(channel) / "bin/cargo"), "vendor", "--locked", "--offline",
                                 "--manifest-path", str(project / "Cargo.toml"), str(tmp)],
                                cwd=ROOT, stdin=subprocess.DEVNULL, capture_output=True, text=True, timeout=120)
        if lock.read_bytes() != pinned:
            raise ValueError("cargo vendor changed the pinned Cargo.lock")
        if result.returncode:
            manifest = " --manifest-path trials/e1-checked/rust/Cargo.toml" if arm == "rust" else ""
            raise ValueError(result.stderr + "\nOne-time setup remedy (needs network): "
                             f"cargo +{channel} fetch --locked{manifest}")
        try:
            os.rename(tmp, dest)
        except OSError:
            if not dest.is_dir():
                raise
        return dest
    finally:
        if tmp.exists():
            shutil.rmtree(tmp)


def environment(config, work):
    return {"PATH": f"{config['sysroot']}/bin:/usr/bin:/bin", "HOME": str(work),
            "CARGO_HOME": str(work / "cargo-home"), "TMPDIR": str(work),
            "CARGO_TARGET_DIR": str(build_dir(config, work)), "CARGO_BUILD_JOBS": "6", "RUST_BACKTRACE": "0"}


def build_dir(config, work):
    return work / "build" if config["sandbox"] else STATE / "rust-build"


def sandbox_command(cmd, config, work, read_only=()):
    args = ["bwrap", "--unshare-all", "--die-with-parent", "--new-session",
            "--cap-drop", "ALL", "--proc", "/proc", "--dev", "/dev"]
    # Opus chose system/store trees to preserve compiler helpers, scripts and symlink chains.
    # Debian's /usr/bin/cc resolves through /etc/alternatives, which holds only symlinks.
    reads = [Path(config["sysroot"]), Path("/usr"), Path("/etc/alternatives"),
             Path("/nix/store"), HERE / "doctest-runner.sh"]
    if config["arm"] == "rust":
        reads.append(Path(config["vendor"]))
    for name in ["/bin", "/lib", "/lib64"]:
        path = Path(name)
        if path.is_symlink():
            args += ["--symlink", str(path.readlink()), name]
        else:
            reads.append(path)
    if config["arm"] == "tide":
        reads.extend([Path(config["driver"]), Path(config["std"])] + [Path(p) for p in config["deps"]])
    reads.extend(Path(p) for p in read_only)
    for path in dict.fromkeys(reads):
        if path.exists(): args += ["--ro-bind", str(path), str(path)]
    return args + ["--bind", str(work), str(work), "--chdir", str(work), "--"] + cmd


def run(cmd, config, work, *, cwd=None, timeout=120, input=None, env=None, read_only=()):
    timeout = min(timeout, config.get("deadline", time.monotonic() + timeout) - time.monotonic())
    if timeout <= 0:
        return {"command": cmd, "code": 124, "stdout": "", "stderr": "whole run time limit exceeded\n"}
    if config["sandbox"]:
        cmd = sandbox_command(cmd, config, work, read_only)
        # bwrap's chdir applies after namespace setup.
        if cwd:
            at = cmd.index("--chdir") + 1
            cmd[at] = str(cwd)
    proc = subprocess.Popen(cmd, cwd=cwd or work, env=environment(config, work) if env is None else env,
                            stdin=subprocess.PIPE if input is not None else subprocess.DEVNULL,
                            stdout=subprocess.PIPE, stderr=subprocess.PIPE, start_new_session=True)
    try:
        out, err = proc.communicate(input, timeout=timeout)
    except subprocess.TimeoutExpired:
        os.killpg(proc.pid, signal.SIGKILL)
        out, err = proc.communicate()
        return {"command": cmd, "code": 124, "stdout": out.decode(errors="replace"),
                "stderr": err.decode(errors="replace") + "\ntime limit exceeded\n"}
    return {"command": cmd, "code": proc.returncode, "stdout": out.decode(errors="replace"),
            "stderr": err.decode(errors="replace")}


def package(work, config, candidate, acceptance=False):
    dest = work / "package"
    dest.mkdir()
    (dest / "src").mkdir()
    task, arm = config["task"], config["arm"]
    entry = "main.rs" if task in BINS else "lib.rs"
    (dest / "src" / entry).write_text(candidate)
    if arm == "rust":
        for name in ["Cargo.toml", "Cargo.lock"]:
            shutil.copyfile(HERE / "rust" / name, dest / name)
        (dest / ".cargo").mkdir()
        (dest / ".cargo/config.toml").write_text('[source.crates-io]\nreplace-with="vendored-sources"\n[source.vendored-sources]\ndirectory=' + json.dumps(config["vendor"]) + '\n')
        if acceptance and task not in BINS:
            (dest / "tests").mkdir()
            (dest / "tests/acceptance.rs").write_text(suite(task, arm))
    return dest


def checks(config, work, pkg):
    if config["arm"] == "rust":
        return [run([config["sysroot"] + "/bin/cargo", "check", "--offline", "--locked", "--all-targets"], config, work, cwd=pkg)]
    entry = pkg / ("src/main.rs" if config["task"] in BINS else "src/lib.rs")
    cmd = [config["driver"], "--edition=2024", "--crate-name=solution",
           "--crate-type=" + ("bin" if config["task"] in BINS else "lib"),
           *tide_artifacts(config)]
    return [run(cmd + ["--out-dir=" + str(work / "output/check-lib"), str(entry)], config, work),
            run(cmd + ["--test", "--out-dir=" + str(work / "output/check-test"), str(entry)], config, work)]


def tide_artifacts(config):
    return ["--tide-std=" + config["std"]] + ["--tide-dep-dir=" + p for p in config["deps"]]


def tide_library_boundary(config, work, pkg, *, link):
    entry = work / "suite/src/lib.rs"
    entry.parent.mkdir(parents=True)
    entry.write_text("extern crate tide_candidate as solution;\n" + suite(config["task"], "tide"))
    out = work / "output/test-acceptance"
    built = run([config["driver"], "--edition=2024", "--test", "--crate-name=acceptance",
                 "--emit=" + ("link" if link else "metadata"), *tide_artifacts(config),
                 "--tide-test-library=" + str(pkg / "src/lib.rs"), "--out-dir=" + str(out), str(entry)],
                config, work)
    results = [dict(built, stage="L")]
    if link and built["code"] == 0:
        results += test_command([str(out / "acceptance-test")], config, work, acceptance=True)
    return results


def executable_tests(config, work, pkg, *, acceptance):
    config = dict(config, deadline=min(config.get("deadline", float("inf")), time.monotonic() + 120))
    if config["arm"] == "tide":
        if acceptance:
            return tide_library_boundary(config, work, pkg, link=True)
        entry = pkg / ("src/main.rs" if config["task"] in BINS else "src/lib.rs")
        out = work / "output/test-writer"
        built = run([config["driver"], "--edition=2024", "--test", "--emit=link", "--crate-name=solution",
                     *tide_artifacts(config), "--out-dir=" + str(out), str(entry)], config, work)
        results = [built]
        if built["code"] == 0:
            results += test_command([str(out / "solution-test")], config, work)
        return results
    cmd = [config["sysroot"] + "/bin/cargo", "test", "--offline", "--locked", "--no-run", "--message-format=json"]
    cmd += ["--test", "acceptance"] if acceptance else (["--bin", "solution"] if config["task"] in BINS else ["--lib"])
    built = run(cmd, config, work, cwd=pkg)
    results = [built]
    if built["code"]: return results
    binaries = []
    for line in built["stdout"].splitlines():
        obj = json.loads(line)
        if obj.get("reason") == "compiler-artifact" and obj.get("executable") and obj["profile"]["test"]:
            if acceptance and (obj["target"]["name"] != "acceptance" or obj["target"]["kind"] != ["test"]
                               or Path(obj["target"]["src_path"]) != pkg / "tests/acceptance.rs"):
                continue
            binaries.append(obj["executable"])
    if acceptance and not binaries:
        results.append({"code": 1, "stdout": "", "stderr": "acceptance build produced no test executable\n"})
    for binary in binaries:
        results += test_command([binary], config, work, acceptance=acceptance)
    if not acceptance and config["task"] not in BINS:
        env = environment(config, work)
        env["RUSTDOCFLAGS"] = "--test-runtool=" + str(HERE / "doctest-runner.sh")
        marker = work / ".doctest-timeout"
        env["E1_DOCTEST_TIMEOUT_FILE"] = str(marker)
        result = run([config["sysroot"] + "/bin/cargo", "test", "--offline", "--locked", "--doc", "--", "--test-threads=1"], config, work, cwd=pkg, env=env)
        if marker.exists():
            result["code"] = 1
            result["stderr"] += "\ndoctest time limit exceeded\n"
        results.append(result)
    return results


def test_command(cmd, config, work, *, acceptance=False):
    # Runtime code sees scratch space and its read-only executable, never package sources.
    with tempfile.TemporaryDirectory(prefix="test-run-", dir=work) as directory:
        execution = Path(directory)
        listing = run(cmd + ["--list", "--format=terse"], config, execution, timeout=120, read_only=[cmd[0]])
        results = [listing]
        if listing["code"]: return results
        tests = [line.removesuffix(": test") for line in listing["stdout"].splitlines() if line.endswith(": test")]
        if acceptance:
            tests = [test for test in tests if "__tide_contract_tests" not in test.split("::")]
            expected = acceptance_tests(config["task"])
            listing["expected_tests"], listing["observed_tests"] = expected, tests
            if tests != expected:
                listing["observed_exit"], listing["code"] = listing["code"], 1
                listing["stderr"] += "\nacceptance test listing differs from frozen suite\n"
                return results
            tests = expected
        for test in tests:
            # Keep the original inode open: candidate path replacement must not redirect the host read.
            with tempfile.NamedTemporaryFile(dir=execution, prefix="libtest-", delete=False) as logfile:
                result = run(cmd + ["--exact", test, "--test-threads=1", "--show-output", "--logfile", logfile.name],
                             config, execution, timeout=10, read_only=[cmd[0]])
                logfile.seek(0)
                record = logfile.read()
            result["observed_exit"] = result["code"]
            result["test_log"] = record.decode("utf-8", errors="replace")
            completed = record == b"ok " + test.encode() + b"\n"
            ignored = not acceptance and re.fullmatch(rb"ignored(?:: [^\r\n]*)? " + re.escape(test.encode()) + rb"\n", record)
            if result["code"] == 0 and not (completed or ignored):
                result["code"] = 1
                result["stderr"] += "\nmissing or inconsistent libtest log record\n"
            results.append(result)
        return results


def binary_cases(config, work, pkg):
    if config["arm"] == "tide":
        out = work / "output/bin"
        built = run([config["driver"], "--edition=2024", "--emit=link", "--crate-type=bin", "--crate-name=solution",
                     *tide_artifacts(config),
                     "--out-dir=" + str(out), str(pkg / "src/main.rs")], config, work)
        binary = out / "solution"
    else:
        built = run([config["sysroot"] + "/bin/cargo", "build", "--offline", "--locked"], config, work, cwd=pkg)
        binary = build_dir(config, work) / "debug/solution"
    results = [built]
    if built["code"] == 0:
        results += run_binary_cases(binary, config, work)
    return results


def run_binary_cases(binary, config, work):
    results = []
    for i, case in enumerate(json.loads((HERE / "suites/binaries.json").read_text())[config["task"]], 1):
        cwd = work / f"case-{i}"
        cwd.mkdir()
        for path, text in case["files"].items():
            (cwd / path).parent.mkdir(parents=True, exist_ok=True)
            (cwd / path).write_bytes(text.encode())
        result = run([str(binary)] + case["args"], config, cwd, timeout=10,
                     input=case["stdin"].encode(), env=case["env"], read_only=[binary])
        good = (result["code"] == case["code"] and result["stdout"] == case["stdout"]
                and result["stderr"].startswith(case["stderr_prefix"]))
        result["observed_exit"] = result["code"]
        result["code"] = 0 if good else 1
        result["case"] = i
        results.append(result)
    return results


def prepare(task, arm, dest, config, reference=False):
    dest.mkdir(parents=True)
    (dest / "src").mkdir()
    (dest / "prompt.md").write_text(task_text(task, arm))
    if arm == "tide":
        for name in ["reference.md", "reference-companion.md"]:
            shutil.copyfile(ROOT / "design" / name, dest / name)
        (dest / "tide.toml").write_text('[package]\nname="e1/solution"\nversion="0.0.0"\ngrants=[]\n')
    else:
        for name in ["Cargo.toml", "Cargo.lock"]:
            shutil.copyfile(HERE / "rust" / name, dest / name)
    entry = dest / ("src/main.rs" if task in BINS else "src/lib.rs")
    entry.write_text(source(task, arm).read_text() if reference else "")
    session = dest.parent / "sessions" / dest.name if reference else STATE / "sessions" / (dest.parent.name + "-" + dest.name)
    session.mkdir(parents=True)
    config = dict(config, task=task, arm=arm, directory=str(dest), runs=0, started=time.time())
    (session / "session.json").write_text(json.dumps(config, indent=2) + "\n")
    return session


def candidate(config):
    directory = Path(config["directory"])
    path = directory / ("src/main.rs" if config["task"] in BINS else "src/lib.rs")
    path.resolve().relative_to(directory.resolve())
    return path.read_bytes()


def candidate_failure(error):
    return {"code": 1, "stdout": "", "stderr": f"candidate source unavailable: {error}\n", "stage": "candidate"}


def command(session, mode):
    with (session / "session.json").open("r+") as lock:
        fcntl.flock(lock, fcntl.LOCK_EX)
        config = json.load(lock)
        now = time.time()
        if config["runs"] >= 20 or (config["started"] is not None and now - config["started"] >= 2400):
            print("E1 budget exhausted", file=sys.stderr)
            return 2
        config["runs"] += 1
        lock.seek(0); lock.truncate(); json.dump(config, lock, indent=2); lock.flush()
        snapshot = session / f"snapshot-{config['runs']:02}"
        snapshot.mkdir()
        raw = b""
        results = []
        try:
            raw = candidate(config)
            text = raw.decode("utf-8")
        except (OSError, ValueError, RuntimeError) as error:
            results.append(candidate_failure(error))
        (snapshot / "source.rs").write_bytes(raw)
        config["deadline"] = time.monotonic() + min(120, 2400 - (now - config["started"]))
        work = snapshot / "run"
        try:
            if not results:
                work.mkdir()
                pkg = package(work, config, text)
                results = checks(config, work, pkg)
                if mode == "test" and all(r["code"] == 0 for r in results):
                    results += executable_tests(config, work, pkg, acceptance=False)
        except (OSError, ValueError, subprocess.SubprocessError) as error:
            results.append({"code": 2, "stdout": "", "stderr": f"E1 run setup error: {error}\n"})
            raise
        finally:
            (snapshot / "results.json").write_text(json.dumps(results, indent=2) + "\n")
            if work.exists():
                shutil.rmtree(work)
        for result in results:
            sys.stdout.write(result["stdout"]); sys.stderr.write(result["stderr"])
        return 0 if all(r["code"] == 0 for r in results) else 1


def grade(config, work, text, compile_only=False):
    acceptance_work = work / "acceptance"
    acceptance_work.mkdir()
    pkg = package(acceptance_work, config, text, acceptance=True)
    results = checks(config, acceptance_work, pkg)
    if compile_only and config["arm"] == "tide" and config["task"] not in BINS:
        results += tide_library_boundary(config, acceptance_work, pkg, link=False)
    # Probe the executable boundary even if metadata fails, so its owner is visible too.
    if not compile_only:
        writer_work = work / "writer"
        writer_work.mkdir()
        writer_pkg = package(writer_work, config, text)
        results += executable_tests(config, writer_work, writer_pkg, acceptance=False)
        results += binary_cases(config, acceptance_work, pkg) if config["task"] in BINS else executable_tests(config, acceptance_work, pkg, acceptance=True)
        flags = integrity_flags(text)
        if flags:
            results.append({"code": 1, "stdout": "", "stderr": "grader review required: " + ", ".join(flags) + "\n",
                            "stage": "integrity", "review_required": True, "flags": flags})
    return {"structure": structure(config["task"], config["arm"], text), "results": results}


def base_config(arm, local=False):
    config = {"arm": arm, "sysroot": str(toolchain(RUST if arm == "rust" else NIGHTLY)), "sandbox": not local}
    if arm == "rust":
        config["vendor"] = str(rust_vendor())
    if arm == "tide":
        vendor = tide_vendor()
        STATE.mkdir(parents=True, exist_ok=True)
        # As rustup's proxy does: build scripts put rustlib/<target>/lib first, whose
        # rustc-dev copy of librustc_driver finds libLLVM only through <sysroot>/lib.
        build_env = {"PATH": config["sysroot"] + "/bin:/usr/bin:/bin", "HOME": str(STATE),
                     "CARGO_HOME": str(STATE / "cargo-home"), "TMPDIR": str(STATE), "CARGO_BUILD_JOBS": os.environ.get("CARGO_BUILD_JOBS", "6"),
                     "LD_LIBRARY_PATH": config["sysroot"] + "/lib"}
        built = subprocess.run([config["sysroot"] + "/bin/cargo", "build", "--offline", "--locked",
                                "-p", "tide-admit", "-p", "tide_std", "--target-dir", str(ROOT / "target"),
                                "--message-format=json", "--config", 'source.crates-io.replace-with="vendored"',
                                "--config", "source.vendored.directory=" + json.dumps(str(vendor))],
                               cwd=ROOT, stdin=subprocess.DEVNULL,
                               capture_output=True, text=True, timeout=120, env=build_env)
        if built.returncode:
            raise ValueError(built.stderr)
        deps = set()
        for line in built.stdout.splitlines():
            obj = json.loads(line)
            if obj.get("reason") != "compiler-artifact": continue
            if obj["target"]["name"] == "tide-admit": config["driver"] = obj["executable"]
            for file in obj["filenames"]:
                if Path(file).suffix in {".rlib", ".so", ".dylib"}:
                    if obj["target"]["name"] == "tide_std": config["std"] = file
                    else: deps.add(str(Path(file).parent))
        config["deps"] = sorted(deps)
    return config


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest="action", required=True)
    render = sub.add_parser("render")
    render.add_argument("task", choices=TASKS); render.add_argument("arm", choices=["rust", "tide"])
    prep = sub.add_parser("prepare")
    prep.add_argument("task", choices=TASKS); prep.add_argument("arm", choices=["rust", "tide"]); prep.add_argument("directory", type=Path)
    cmd = sub.add_parser("command")
    cmd.add_argument("session", type=Path); cmd.add_argument("mode", choices=["check", "test"])
    grader = sub.add_parser("grade")
    grader.add_argument("session", type=Path)
    grader.add_argument("--first", action="store_true", help="grade the first saved source instead of the final file")
    grader.add_argument("--compile-only", action="store_true", help="acceptance compilation for the first-check measure")
    refs = sub.add_parser("references")
    refs.add_argument("arm", choices=["rust", "tide"])
    refs.add_argument("tasks", nargs="*", choices=TASKS)
    refs.add_argument("--local", action="store_true", help="readiness only: run known reference sources without namespace isolation")
    args = parser.parse_args()
    if args.action == "render": print(task_text(args.task, args.arm), end=""); return 0
    if args.action == "prepare":
        print(prepare(args.task, args.arm, args.directory.resolve(), base_config(args.arm))); return 0
    if args.action == "command": return command(args.session.resolve(), args.mode)
    if args.action == "grade":
        config = json.loads((args.session / "session.json").read_text())
        failures = []
        try:
            if args.first:
                saved = json.loads((args.session / "snapshot-01/results.json").read_text())
                failures = [r for r in saved if r.get("stage") == "candidate"]
                raw = (args.session / "snapshot-01/source.rs").read_bytes()
            else:
                raw = candidate(config)
            if not failures:
                text = raw.decode("utf-8")
        except (OSError, ValueError, RuntimeError) as error:
            failures = [candidate_failure(error)]
        if failures:
            result = {"structure": False, "results": failures}
        else:
            with tempfile.TemporaryDirectory(prefix="grade-", dir=STATE) as directory:
                result = grade(config, Path(directory), text, args.compile_only)
        (args.session / ("first-grade.json" if args.first else "final-grade.json")).write_text(json.dumps(result, indent=2) + "\n")
        return 0 if (args.compile_only or result["structure"]) and all(r["code"] == 0 for r in result["results"]) else 1
    config = base_config(args.arm, args.local)
    STATE.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix=f"{args.arm}-", dir=STATE) as directory:
        root = Path(directory)
        records = []
        for task in args.tasks or TASKS:
            current = dict(config, task=task)
            task_dir = root / task
            session = prepare(task, args.arm, task_dir, current, reference=True)
            checked = subprocess.run([sys.executable, str(Path(__file__).resolve()), "command", str(session), "check"],
                                     cwd=STATE, stdin=subprocess.DEVNULL, capture_output=True, timeout=180)
            work = root / f"{task}-grade"
            work.mkdir()
            record = dict(task=task, check=checked.returncode, **grade(current, work, source(task, args.arm).read_text()))
            record["check_stdout"] = checked.stdout.decode(errors="replace")
            record["check_stderr"] = checked.stderr.decode(errors="replace")
            record["passed"] = record["check"] == 0 and record["structure"] and all(r["code"] == 0 for r in record["results"])
            records.append(record)
            print(task, "PASS" if record["passed"] else "FAIL", flush=True)
        report = {"arm": args.arm, "sandbox": config["sandbox"], "rustc": subprocess.check_output([config["sysroot"] + "/bin/rustc", "-Vv"], text=True), "records": records}
        (STATE / f"{args.arm}-results.json").write_text(json.dumps(report, indent=2) + "\n")
        return 0 if all(r["passed"] for r in records) else 1


if __name__ == "__main__":
    try:
        sys.exit(main())
    except (OSError, ValueError, subprocess.SubprocessError) as error:
        print(f"E1 setup error: {error}", file=sys.stderr)
        sys.exit(2)
