#!/usr/bin/env python3
"""Bounded E1 batches. Raw writer/task/source data stays in the external run directory."""
import argparse
from concurrent.futures import ThreadPoolExecutor
import fcntl
import json
import os
from pathlib import Path
import queue
import random
import re
import shutil
import signal
import subprocess
import sys
import threading
import tempfile
import time

import gate_checks as g
import harness as h

SEED = 20261031
FAULTS = {"init_failure", "cli_error", "broker_crash", "harness_setup", "host_interruption", "log_audit"}


def order():
    slots = sorted((task, arm, replicate) for task in h.TASKS for arm in ["rust", "tide"] for replicate in [1, 2])
    random.Random(SEED).shuffle(slots)
    return slots


def order_hash(slots):
    return h.sha256(json.dumps(slots).encode()).hexdigest()


def jsonl(path):
    return [json.loads(line) for line in path.read_text().splitlines()] if path.exists() else []


class RunCancelled(RuntimeError): pass
class ConfinementBreach(RuntimeError): pass


def writer_events(path, killed):
    raw = path.read_bytes()
    truncated = killed and bool(raw) and not raw.endswith(b"\n")
    if truncated: raw = raw.rpartition(b"\n")[0]
    return [json.loads(line) for line in raw.splitlines()], truncated


def init_assertion(event, model):
    if not isinstance(event, dict): raise ValueError("init must be an object")
    servers = event.get("mcp_servers", [])
    if (event.get("type") != "system" or event.get("subtype") != "init"
        or set(event.get("tools", [])) != g.TOOLS or len(event.get("tools", [])) != len(g.TOOLS)
        or len(servers) != 1 or not isinstance(servers[0], dict) or servers[0].get("name") != "e1" or servers[0].get("status") != "connected"
        or event.get("plugins") != [] or event.get("model") != model):
        raise ValueError("init tool/server/plugin/model assertion failed")


def blocks(value):
    if isinstance(value, dict):
        yield value
        for child in value.values(): yield from blocks(child)
    elif isinstance(value, list):
        for child in value: yield from blocks(child)


PROBE_CONTENT = "E1 confinement probe\n"
DENIAL_REASON = "--restricted: path outside the working directory"


def permission_denied(use, result, events):
    name = use.get("name")
    if result.get("is_error") is not True: return False
    content = result.get("content", "")
    if isinstance(content, list):
        content = "\n".join(part.get("text", "") for part in content if isinstance(part, dict) and part.get("type") == "text")
    return isinstance(content, str) and any(
        event.get("type") == "system" and event.get("subtype") == "permission_denied"
        and event.get("tool_name") == name and event.get("tool_use_id") == use.get("id")
        and event.get("decision_reason_type") == "other" and event.get("decision_reason") == DENIAL_REASON
        and isinstance(event.get("message"), str) and bool(event["message"]) and event["message"] in content
        for event in events if isinstance(event, dict))


def probe_denied(use, result, events):
    name, args = use.get("name"), use.get("input", {})
    if not isinstance(args, dict) or not isinstance(args.get("file_path"), str) or not Path(args["file_path"]).is_absolute():
        return False
    valid = ((name == "Read" and set(args) == {"file_path", "offset", "limit"}
              and type(args["offset"]) is int and args["offset"] == 1
              and type(args["limit"]) is int and args["limit"] == 1)
             or (name == "Write" and set(args) == {"file_path", "content"} and args["content"] == PROBE_CONTENT))
    return valid and permission_denied(use, result, events)


def audit(events, directory, model, probe=False):
    uses, results, models = [], {}, []
    usage, peak, turns, reads = {}, 0, 0, []
    messages = {}
    for event in events:
        if not isinstance(event, dict): raise ValueError("log event must be an object")
        if event.get("type") == "assistant":
            message = event.get("message", {})
            if not isinstance(message, dict): raise ValueError("assistant message must be an object")
            messages[message.get("id", str(len(messages)))] = message
        for block in blocks(event):
            if block.get("type") == "tool_use": uses.append(block)
            if block.get("type") == "tool_result": results[block.get("tool_use_id")] = block
    for message in messages.values():
        turns += 1
        models.append(message.get("model"))
        current = message.get("usage", {})
        for name, count in current.items():
            if isinstance(count, int): usage[name] = usage.get(name, 0) + count
        peak = max(peak, sum(current.get(name, 0) for name in
                   ["input_tokens", "cache_read_input_tokens", "cache_creation_input_tokens"]))
    errors, denied_outside, breaches = [], [], []
    for use in uses:
        name, args = use.get("name"), use.get("input", {})
        result = results.get(use.get("id"), {})
        if not isinstance(args, dict):
            if name in g.TOOLS and result.get("is_error") is True: continue
            errors.append("tool input must be an object"); continue
        if name not in g.TOOLS:
            errors.append("forbidden tool: " + str(name)); continue
        if name.startswith("mcp__"):
            if args != {} and result.get("is_error") is not True: errors.append("broker arguments")
            continue
        path = args.get("file_path")
        if not isinstance(path, str):
            if result.get("is_error") is True: continue
            errors.append("missing file path"); continue
        resolved = (directory / path).resolve()
        if not resolved.is_relative_to(directory.resolve()):
            result = results.get(use.get("id"), {})
            if permission_denied(use, result, events):
                denied_outside.append({"tool": name, "path": path, "tool_use_id": use.get("id")})
                if probe and not probe_denied(use, result, events): errors.append("invalid probe request")
            else:
                breaches.append({"tool": name, "path": path, "tool_use_id": use.get("id")})
        if name == "Read" and resolved.name in {"reference.md", "reference-companion.md"}:
            offset, limit = args.get("offset", 1), args.get("limit")
            sections = []
            trusted = h.ROOT / "design" / resolved.name
            if trusted.exists():
                headings = [(i, match[1]) for i, line in enumerate(trusted.read_text().splitlines(), 1)
                            if (match := re.match(r"^#+ (\d+(?:\.\d+)*)\b", line))]
                for index, (start, section) in enumerate(headings):
                    stop = headings[index + 1][0] if index + 1 < len(headings) else float("inf")
                    if start < (offset + limit if limit else float("inf")) and stop > offset:
                        sections.append(section)
            reads.append({"document": resolved.name, "offset": offset, "limit": limit,
                          "sections": sections, "tool_use_id": use.get("id")})
    if any(model_id != model for model_id in models): errors.append("assistant model changed")
    return {"passed": not errors and not breaches, "errors": errors, "confinement_breach": breaches,
            "denied_outside": denied_outside, "turns": turns, "tokens": usage,
            "peak_context": peak, "reference_reads": reads, "models": models,
            "api_retry_events": [event for event in events if event.get("type") == "system" and event.get("subtype") == "api_retry"]}


# CLI 2.1.321: --safe-mode also drops --mcp-config servers, so the writer runs without it. These
# settings, WRITER_ENV and --disable-slash-commands take over what it disabled: the builtin
# plugins the pinned CLI loads, hooks, CLAUDE.md, auto-memory and claude.ai connectors (which
# --strict-mcp-config does not cover and which load after init, out of the init assertion's
# sight). The init assertion still requires plugins == [], so a plugin added by another CLI
# fails init rather than loading.
BUILTIN_PLUGINS = ["cc-plugin-agents-md", "cc-plugin-plugin-authoring", "cc-plugin-privacy", "cc-plugin-telemetry"]
WRITER_SETTINGS = {"disableAllHooks": True,
                   "enabledPlugins": {f"{name}@builtin": False for name in BUILTIN_PLUGINS}}
WRITER_ENV = {"DISABLE_AUTOUPDATER": "1", "MAX_MCP_OUTPUT_TOKENS": "2000000",
              "CLAUDE_CODE_DISABLE_CLAUDE_MDS": "1", "CLAUDE_CODE_DISABLE_AUTO_MEMORY": "1",
              "ENABLE_CLAUDEAI_MCP_SERVERS": "0"}


def writer_argv(cli, model, mcp):
    return [cli, "-p", "--model", model, "--restricted", "--tools", "Read,Write,Edit",
            "--strict-mcp-config", "--mcp-config", str(mcp), "--allowedTools", *sorted(g.TOOLS),
            "--permission-mode", "dontAsk", "--permission-prompts", "none", "--disable-slash-commands",
            "--settings", json.dumps(WRITER_SETTINGS, sort_keys=True),
            "--no-session-persistence", "--output-format", "stream-json", "--verbose"]


def mcp_config(session):
    config = {"mcpServers": {"e1": {"command": sys.executable,
              "args": [str(h.HERE / "broker.py"), str(session)]}}}
    g.save(session / "mcp.json", config)
    return session / "mcp.json"


def launch_clock(session, seconds=None):
    with (session / "session.json").open("r+") as stream:
        fcntl.flock(stream, fcntl.LOCK_EX)
        config = json.load(stream)
        config.update(started=time.time(), started_monotonic=time.monotonic())
        if config.get("dry_run") and seconds is not None: config["session_seconds"] = seconds
        config["session_seconds"] = h.session_limit(config)
        stream.seek(0); stream.truncate(); json.dump(config, stream); stream.flush()
    return config


ACTIVE = {}
ACTIVE_LOCK = threading.Lock()


def process_identity(pid):
    fields = Path(f"/proc/{pid}/stat").read_text().rsplit(")", 1)[1].split()
    return {"pid": pid, "start": fields[19], "group": int(fields[2]), "state": fields[0],
            "boot": Path("/proc/sys/kernel/random/boot_id").read_text().strip()}


def group_alive(group):
    for path in Path("/proc").iterdir():
        if not path.name.isdigit(): continue
        try: current = process_identity(int(path.name))
        except (FileNotFoundError, ProcessLookupError): continue
        if current["group"] == group and current["state"] != "Z": return True
    return False


def wait_quiescent(session):
    path = session / "process.json"
    if path.exists():
        saved = json.loads(path.read_text())
        boot = Path("/proc/sys/kernel/random/boot_id").read_text().strip()
        if saved["boot"] == boot:
            until = time.monotonic() + 10
            while True:
                try: current = process_identity(saved["pid"])
                except (FileNotFoundError, ProcessLookupError): current = None
                if current is None or any(current[key] != saved[key] for key in ["start", "group", "boot"]):
                    break
                if saved["group"] != saved["pid"]: raise ValueError("invalid saved guard group")
                if not group_alive(saved["group"]): break
                if time.monotonic() >= until: raise ValueError("previous writer group is still alive")
                time.sleep(0.02)


def reclaim_writer(session):
    # The parent-death guard stops the old group; recovery only observes it.
    for directory in [session, *session.glob("*-grader")]:
        wait_quiescent(directory)
    # A detached harness command can outlive its killed broker, but holds this lock.
    if (session / "session.json").exists():
        with (session / "session.json").open() as lock:
            until = time.monotonic() + 160
            while True:
                try:
                    fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB); break
                except BlockingIOError:
                    if time.monotonic() >= until: raise ValueError("previous harness command did not stop")
                    time.sleep(0.05)


def signal_guard(handle):
    # Serialize fd close with signalling, even if another thread already reaped the guard.
    with ACTIVE_LOCK:
        if not handle.closed:
            try: signal.pidfd_send_signal(handle.fileno(), signal.SIGTERM)
            except ProcessLookupError: pass
            except OSError as error:
                raise RuntimeError("cannot signal guard safely; refusing further attempts") from error


def release_guard(proc, handle):
    with ACTIVE_LOCK:
        if ACTIVE.get(proc.pid) is handle: ACTIVE.pop(proc.pid)
        handle.close()


def end_guard(proc, handle):
    signal_guard(handle)
    try: proc.wait(timeout=10)
    except subprocess.TimeoutExpired as error:
        raise RuntimeError("guard did not stop; refusing further attempts") from error
    release_guard(proc, handle)


def stop_active():
    with ACTIVE_LOCK: handles = list(ACTIVE.values())
    for handle in handles: signal_guard(handle)


def start_guard(argv, session, *, cancelled=None, **kwargs):
    session.mkdir(parents=True, exist_ok=True)
    if not (session / "launch.json").exists(): g.save(session / "launch.json", {"argv": argv})
    read_grant, write_grant = os.pipe()
    proc = handle = None
    try:
        with ACTIVE_LOCK:
            if cancelled is not None and cancelled.is_set(): raise InterruptedError("coordinator shutdown")
            proc = subprocess.Popen([sys.executable, str(h.HERE / "writer_guard.py"), str(os.getpid()),
                str(read_grant), str(session), *argv], start_new_session=True, pass_fds=(read_grant,), **kwargs)
            handle = os.fdopen(os.pidfd_open(proc.pid), "rb", buffering=0)
            ACTIVE[proc.pid] = handle
        g.save(session / "process.json", process_identity(proc.pid))
        return proc, handle, write_grant
    except BaseException:
        os.close(write_grant)  # Without a grant the guard cannot launch its child.
        if handle is not None: end_guard(proc, handle)
        elif proc is not None: proc.wait(timeout=10)
        raise
    finally:
        os.close(read_grant)


def classify_session(session, directory, model, *, reason="early_stop", fault=None, evidence=None,
                     killed=False, writer_exit=-signal.SIGKILL, probe=False):
    budget_path = session / "budget.json"
    if budget_path.exists():
        budget = json.loads(budget_path.read_text())
        reason = budget["reason"]
        if budget.get("infrastructure"): fault, evidence = "harness_setup", "last command setup failure"
    initialized, events = False, []
    try:
        events, truncated_tail = writer_events(session / "writer.jsonl", killed) if (session / "writer.jsonl").exists() else ([], False)
        if events:
            init_assertion(events[0], model)
            initialized = True
        checked = audit(events, directory, model, probe)
    except (ValueError, TypeError, KeyError) as error:
        checked = {"passed": False, "errors": [str(error)]}
        truncated_tail = False
    if checked.get("confinement_breach"):
        g.save(session / "confinement-breach.json", checked)
        raise ConfinementBreach("outside access was not denied; stop run for operator")
    if not initialized and fault is None: fault, evidence = "init_failure", "missing init"
    if not checked["passed"] and fault != "init_failure": fault, evidence = "log_audit", checked["errors"]
    result_events = [event for event in events if isinstance(event, dict) and event.get("type") == "result"]
    exit_record = session / "process-exit.json"
    if exit_record.exists(): writer_exit = json.loads(exit_record.read_text())["exit"]
    if fault is None and reason not in {"time_limit", "run_limit"}:
        if writer_exit or any(event.get("is_error") or event.get("subtype", "").startswith("error") for event in result_events):
            fault, evidence = "cli_error", {"exit": writer_exit, "result": result_events[-1:]}
        elif not result_events:
            fault, evidence = "cli_error", "missing result event"
        else: reason = "done"
    try: broker = jsonl(session / "broker.jsonl")
    except ValueError:
        broker = [{"fault": "broker_crash"}]
    if initialized and fault is None and reason not in {"time_limit", "run_limit"}:
        until = time.monotonic() + 1
        while any(row.get("event") == "start" for row in broker) and not any(row.get("event") == "stop" for row in broker) and time.monotonic() < until:
            time.sleep(0.02)
            broker = jsonl(session / "broker.jsonl")
        if not any(row.get("event") == "start" for row in broker) or not any(row.get("event") == "stop" for row in broker):
            fault, evidence = "broker_crash", "missing broker lifecycle record"
    if any(row.get("fault") for row in broker): fault, evidence = "broker_crash", "broker.jsonl"
    if any(row.get("exit") == 2 for row in broker): fault, evidence = "harness_setup", "broker exit 2"
    return {"end_reason": fault or reason, "fault": fault, "evidence": evidence, "audit": checked,
            "truncated_tail": truncated_tail, "exit": writer_exit,
            "truncations": sum(row.get("truncated", False) for row in broker)}


def run_writer(session, cli, model, dry=False, seconds=None, prompt=None, probe=False, on_started=lambda: None, cancelled=None):
    if not dry: g.managed_config_absent()
    config = json.loads((session / "session.json").read_text())
    directory = Path(config["directory"])
    mcp = mcp_config(session)
    home = session / "config-home"
    home.mkdir()
    env = {"HOME": str(home), "CLAUDE_CONFIG_DIR": str(home), "PATH": os.environ.get("PATH", "/usr/bin:/bin"),
           **WRITER_ENV}
    for key in ["ANTHROPIC_API_KEY", "CLAUDE_CODE_OAUTH_TOKEN"]:
        if key in os.environ: env[key] = os.environ[key]
    argv = ([sys.executable, str(h.HERE / "standin.py"), str(session), str(mcp)] if dry
            else writer_argv(cli, model, mcp))
    g.save(session / "launch.json", {"argv": argv, "model": model, "effort": "default (flag not passed)", "dry_run": dry})
    with (session / "writer.jsonl").open("wb") as log, (session / "writer.stderr").open("w") as err, \
         tempfile.TemporaryFile(mode="w+") as special_prompt:
        if prompt is not None:
            special_prompt.write(prompt); special_prompt.seek(0)
        with (directory / "prompt.md").open() as task_prompt:
            proc, handle, write_grant = start_guard(argv, session, cancelled=cancelled, cwd=directory, env=env,
                stdin=task_prompt if prompt is None else special_prompt, stdout=subprocess.PIPE, stderr=err)
            try:
                started = launch_clock(session, seconds)
                seconds = started["session_seconds"]
                with ACTIVE_LOCK:
                    if cancelled is not None and cancelled.is_set(): raise RunCancelled("run cancelled before grant")
                    os.write(write_grant, b"g")
            except BaseException:
                end_guard(proc, handle)
                raise
            finally:
                os.close(write_grant)
        lines = queue.Queue()
        def reader():
            for line in proc.stdout:
                log.write(line); log.flush(); lines.put(line)
            lines.put(None)
        thread = threading.Thread(target=reader, daemon=True)
        thread.start()
        reason, fault, evidence, initialized, killed = "early_stop", None, None, False, False
        try:
            on_started()
            while True:
                if cancelled is not None and cancelled.is_set():
                    fault, evidence = "host_interruption", "coordinator shutdown"; break
                if (session / "budget.json").exists():
                    budget = json.loads((session / "budget.json").read_text())
                    try: completed = jsonl(session / "broker.jsonl")
                    except ValueError: completed = []  # An append can be observed before its final newline.
                    if any(row.get("completed_run") == budget["completed_run"] for row in completed):
                        reason = budget["reason"]
                        break
                left = seconds - (time.monotonic() - started["started_monotonic"])
                if left <= 0:
                    reason = "time_limit"; break
                try: line = lines.get(timeout=min(left, 0.05))
                except queue.Empty:
                    continue
                if line is None:
                    proc.wait(timeout=max(0.01, left))
                    break
                try:
                    event = json.loads(line)
                    if not isinstance(event, dict): raise ValueError("log event must be an object")
                    if not initialized:
                        init_assertion(event, model)
                        initialized = True
                except (ValueError, TypeError, KeyError) as error:
                    fault, evidence = "init_failure", str(error); break
        except subprocess.TimeoutExpired:
            reason = "time_limit"
        except (KeyboardInterrupt, SystemExit):
            fault, evidence = "host_interruption", "coordinator interrupted"
        finally:
            killed = reason in {"run_limit", "time_limit"} and proc.poll() is None
            end_guard(proc, handle)
            thread.join(timeout=10)
            proc.stdout.close()
            if thread.is_alive(): raise ValueError("writer log pipe did not close")
    result = classify_session(session, directory, model, reason=reason, fault=fault, evidence=evidence,
                              killed=killed, writer_exit=proc.returncode, probe=probe)
    drift = time.time() - started["started"] - (time.monotonic() - started["started_monotonic"])
    return dict(result, wall_seconds=time.monotonic() - started["started_monotonic"],
                clock_drift=drift, clock_drift_flag=abs(drift) > 1)



def freeze_final(session):
    with (session / "session.json").open() as stream:
        fcntl.flock(stream, fcntl.LOCK_EX)
        config = json.load(stream)
        h.capture_final(session, config)
        g.save(session / "session.json", config)
        directory = Path(config["directory"])
        for path in directory.rglob("*"):
            if not path.is_symlink(): path.chmod(0o555 if path.is_dir() else 0o444)
        directory.chmod(0o555)
    return g.digest(session / "final/source.rs")


def freeze_and_grade(session, cancelled=None):
    freeze_final(session)
    grades = {}
    for name, flags in [("first", ["--first", "--compile-only"]), ("final", [])]:
        with (session / (name + "-grade.stdout")).open("w") as out, (session / (name + "-grade.stderr")).open("w") as err:
            guard_dir = session / (name + "-grader")
            (guard_dir / "process-exit.json").unlink(missing_ok=True)
            (session / (name + "-grade.json")).unlink(missing_ok=True)
            proc, handle, grant = start_guard([sys.executable, str(h.HERE / "harness.py"), "grade", str(session), *flags],
                guard_dir, cancelled=cancelled, stdin=subprocess.DEVNULL, stdout=out, stderr=err)
            try:
                with os.fdopen(grant, "wb", buffering=0) as launch:
                    with ACTIVE_LOCK:
                        if cancelled is not None and cancelled.is_set(): raise RunCancelled("run cancelled before grader grant")
                        launch.write(b"g")
                try:
                    code = proc.wait(timeout=600)
                    exited = guard_dir / "process-exit.json"
                    if exited.exists(): code = json.loads(exited.read_text())["exit"]
                except subprocess.TimeoutExpired: code = 2
            finally:
                end_guard(proc, handle)
            record = session / (name + "-grade.json")
            infrastructure = code not in (0, 1)
            try:
                data = json.loads(record.read_text())
                infrastructure |= data.get("infrastructure", False) or any(row.get("infrastructure") for row in data["results"])
            except (OSError, ValueError, KeyError, TypeError, AttributeError):
                infrastructure = True
            grades[name] = {"exit": code, "record": str(record), "infrastructure": infrastructure}
            if infrastructure: break
    return grades


def grade_frozen(session, cancelled=None):
    path = session / "grading.json"
    history = json.loads(path.read_text()) if path.exists() else []
    def archive(index):
        dest = session / f"grading-{index}"
        dest.mkdir(exist_ok=True)
        for name in ["first", "final"]:
            for suffix in ["json", "stdout", "stderr"]:
                source = session / f"{name}-grade.{suffix}"
                if source.exists(): shutil.copyfile(source, dest / source.name)
    if history and not history[-1].get("complete"):
        reclaim_writer(session)
        archive(len(history))
        history[-1] = {"complete": True, "infrastructure": True, "error": "grading interrupted"}
        g.save(path, history)
    while len(history) < 3:
        if cancelled is not None and cancelled.is_set(): raise RunCancelled("run cancelled before grading")
        history.append({"complete": False})
        g.save(path, history)
        try: grades = freeze_and_grade(session, cancelled)
        except (OSError, ValueError, subprocess.SubprocessError) as error:
            grades = {"setup": {"exit": 2, "infrastructure": True, "error": str(error)}}
        archive(len(history))
        infrastructure = any(row.get("infrastructure") or row.get("exit") not in (0, 1) for row in grades.values())
        history[-1] = {"complete": True, "infrastructure": infrastructure, "grades": grades}
        g.save(path, history)
        if cancelled is not None and cancelled.is_set(): raise RunCancelled("run cancelled during grading")
        if not infrastructure: return grades
    raise RuntimeError("frozen snapshot grading failed after two retries; stop run for operator")


def run_slot(slot, triple, attempt_runner, record, cancelled=None):
    attempts = []
    for attempt in [1, 2]:
        if cancelled is not None and cancelled.is_set(): raise RunCancelled("run cancelled before next attempt")
        result = attempt_runner(attempt)
        if not result.get("fault") and "grades" in result and any(
                grade.get("infrastructure") or grade.get("exit") not in (0, 1) for grade in result["grades"].values()):
            raise RuntimeError("grading infrastructure must retry the frozen snapshot, never the writer")
        if result.get("session") and Path(result["session"]).exists():
            g.save(Path(result["session"]) / "trial.json", {"slot": slot, "triple": triple, **result})
        if result.get("fault") and "grades" in result:
            raise ValueError("faulted attempts must never be graded")
        if result.get("fault") is not None and result["fault"] not in FAULTS:
            raise ValueError("unclassified infrastructure fault")
        attempts.append(result)
        record({"slot": slot, "triple": triple, "attempts": attempts, "complete": not result.get("fault") or attempt == 2,
                "reruns": len(attempts) - 1, "failed_infrastructure": bool(result.get("fault") and attempt == 2),
                "first_pass": False if result.get("fault") else result.get("grades", {}).get("first", {}).get("exit") == 0,
                "final_pass": False if result.get("fault") else result.get("grades", {}).get("final", {}).get("exit") == 0})
        if not result.get("fault") or (cancelled is not None and cancelled.is_set()): break
    return attempts


def prerequisites(root, state):
    if state["dry_run"]: return
    current = g.canary_key()
    for arm in ["rust", "tide"]:
        record = json.loads((root / (arm + "-canary.json")).read_text())
        if record.get("key") != current or not record.get("passed") or record.get("arm") != arm:
            raise ValueError("missing or stale canary: " + arm)
    probe = json.loads((root / "probe.json").read_text())
    if not probe.get("passed") or probe.get("key") != current or probe.get("writer") != state["identity"]["cheap"]:
        raise ValueError("missing or stale tool-surface probe")
    before = json.loads((root / "store-before.json").read_text())
    if not before.get("passed") or before.get("key") != current:
        raise ValueError("missing or failed pre-run Nix-store scan")


def batch(root, state, count):
    prerequisites(root, state)
    pending = [i for i in range(len(state["order"])) if not (root / f"slot-{i:03}.json").exists()
               or not json.loads((root / f"slot-{i:03}.json").read_text()).get("complete")]
    selected = pending[:count]
    launch_condition, launched, cancelled = threading.Condition(), set(), threading.Event()
    def cancel_run():
        with ACTIVE_LOCK: cancelled.set()
        with launch_condition: launch_condition.notify_all()
    if list(root.glob("sessions/*/confinement-breach.json")):
        raise ConfinementBreach("run has a confinement breach; operator action required")
    def execute(i):
        triple = state["order"][i]
        path = root / f"slot-{i:03}.json"
        old = json.loads(path.read_text()) if path.exists() else {}
        def started(session):
            return (session / "launch.json").exists() and (session / "process.json").exists()
        def mark_launched():
            with launch_condition:
                launched.add(i); launch_condition.notify_all()
        def finish(session, result):
            if result.get("fault"): return result
            final_hash = freeze_final(session)
            if (session / "writer-result.json").exists() and result.get("final_sha256") != final_hash:
                raise RuntimeError("final snapshot hash changed; stop run for operator")
            result["final_sha256"] = final_hash
            # The writer's ended result is durable before any grading starts.
            g.save(session / "writer-result.json", result)
            g.save(session / "trial.json", {"slot": i, "triple": triple, **result})
            result["grades"] = grade_frozen(session, cancelled)
            final_grade = session / "final-grade.json"
            result["integrity_flags"] = [flag for row in json.loads(final_grade.read_text())["results"]
                                         for flag in row.get("flags", [])] if final_grade.exists() else []
            first = session / "snapshot-01/source.rs"
            raw = first.read_bytes() if first.exists() else b""
            result["first_snapshot"] = {"present": first.exists(), "bytes": len(raw),
                                        "holes": len(re.findall(rb"\b(?:todo|hole)!", raw))}
            return result
        def attempt_runner(attempt):
            if cancelled.is_set(): raise RunCancelled("run cancelled before next attempt")
            session = root / "sessions" / f"{i:03}-{attempt}"
            directory = root / "w" / f"{i:03}-{attempt}"
            previous = old.get("attempts", [])[attempt - 1] if attempt <= len(old.get("attempts", [])) else None
            result_path = session / "writer-result.json"
            if (session / "final").exists() and (result_path.exists() or not (
                    previous and previous.get("fault") and previous.get("end_reason") != "host_interruption")):
                reclaim_writer(session)
                if result_path.exists(): result = json.loads(result_path.read_text())
                else:
                    result = classify_session(session, directory, state["model"], killed=(session / "budget.json").exists())
                    result.update(session=str(session), attempt=attempt, dry_run=state["dry_run"],
                                  runs=json.loads((session / "session.json").read_text())["runs"],
                                  denied_outside_by_arm={triple[1]: len(result["audit"].get("denied_outside", []))})
                result.pop("grades", None)
                mark_launched()
                return finish(session, result)
            if started(session):
                reclaim_writer(session)
                mark_launched()
                if previous and not previous.get("fault"): return finish(session, previous)
                return previous or {"fault": "host_interruption", "end_reason": "host_interruption",
                                    "session": str(session), "attempt": attempt, "dry_run": state["dry_run"]}
            # Preserve partial preparation, but an ungranted guard is not an attempt.
            if directory.exists() or session.exists():
                dest = root / "unstarted" / f"{i:03}-{attempt}-{time.time_ns()}"
                dest.mkdir(parents=True)
                if directory.exists(): directory.rename(dest / "writer")
                if session.exists(): session.rename(dest / "session")
            if not state["dry_run"] and g.cheap(state["configs"], state["cli"], state["model"]) != state["identity"]["cheap"]:
                raise RuntimeError("identity changed; stop run")
            config = dict(state["configs"][triple[1]], dry_run=state["dry_run"], writer_model=state["model"])
            h.prepare(triple[0], triple[1], directory, config, session=session)
            if not state["dry_run"] and g.cheap(state["configs"], state["cli"], state["model"]) != state["identity"]["cheap"]:
                raise RuntimeError("identity changed during preparation; stop run")
            with launch_condition:
                launch_condition.wait_for(lambda: cancelled.is_set() or all(j in launched for j in selected if j < i))
                if cancelled.is_set(): raise RunCancelled("batch stopped before launch")
            granted = False
            def on_started():
                nonlocal granted
                granted = True
                history = json.loads(path.read_text()).get("attempts", [])[:attempt - 1] if path.exists() else []
                g.save(path, {"complete": False, "dry_run": state["dry_run"], "attempts": history + [{
                    "fault": "host_interruption", "end_reason": "host_interruption", "session": str(session),
                    "attempt": attempt, "dry_run": state["dry_run"], "evidence": "unfinished granted writer"}]})
                mark_launched()
            try:
                result = run_writer(session, state["cli"], state["model"], state["dry_run"], state["seconds"], on_started=on_started, cancelled=cancelled)
            except (OSError, ValueError, subprocess.SubprocessError) as error:
                if not granted: raise RuntimeError("writer setup failed before grant") from error
                result = {"fault": "cli_error", "end_reason": "cli_error", "evidence": str(error)}
            result.update(session=str(session), attempt=attempt, dry_run=state["dry_run"])
            result["runs"] = json.loads((session / "session.json").read_text())["runs"]
            result["denied_outside_by_arm"] = {triple[1]: len(result.get("audit", {}).get("denied_outside", []))}
            if state["dry_run"] and not result["fault"] and result["end_reason"] == "run_limit":
                from standin import Broker
                broker = Broker(session / "mcp.json")
                try: refused = broker.tool("check")
                finally: broker.close()
                if refused != 3: raise RuntimeError("dry-run broker accepted call after exhaustion")
                result["run_21_refused"] = True
            return finish(session, result)
        attempts = run_slot(i, triple, attempt_runner, lambda row: g.save(path, dict(row, dry_run=state["dry_run"])), cancelled)
        mark_launched()
        last = attempts[-1]
        print(i, last["end_reason"], "runs", last.get("runs", 0), "grades", last.get("grades", {}), flush=True)
    def guarded(i):
        try: return execute(i)
        except BaseException:
            cancel_run()
            raise
    pool = ThreadPoolExecutor(max_workers=state["jobs"])
    try:
        list(pool.map(guarded, selected))
    except BaseException:
        cancel_run()
        stop_active()
        raise
    finally:
        pool.shutdown(wait=True, cancel_futures=True)
    if cancelled.is_set(): raise RunCancelled("run cancelled")
    if all((root / f"slot-{i:03}.json").exists() and json.loads((root / f"slot-{i:03}.json").read_text()).get("complete") for i in range(len(state["order"]))):
        if not state["dry_run"]:
            record = g.store_scan(); record["key"] = g.canary_key(); g.save(root / "store-after.json", record)
            if not record["passed"]: raise ValueError("post-run store scan failed; gate invalid")


def probe(root, state):
    if state["dry_run"]: raise ValueError("probe requires the recorded real CLI")
    g.managed_config_absent()  # before any probe directory exists, so a refused probe can rerun
    session = root / "probe-session"
    directory = root / "probe-writer"
    directory.mkdir()
    (directory / "prompt.md").write_text("")
    session.mkdir()
    g.save(session / "session.json", {"directory": str(directory), "started": None, "runs": 0})
    outside = root / "probe-outside"
    requests = [{"name": "Read", "input": {"file_path": str(path), "offset": 1, "limit": 1}}
                for path in [h.ROOT / "AGENTS.md", Path("/etc/hostname")]]
    requests.append({"name": "Write", "input": {"file_path": str(outside), "content": PROBE_CONTENT}})
    if not all(Path(item["input"]["file_path"]).is_file() for item in requests[:2]) or outside.exists() or outside.is_symlink():
        raise ValueError("probe targets must be existing readable files and a fresh outside write path")
    controls = [{"name": "Write", "input": {"file_path": str(directory / "inside.txt"), "content": "control\n"}},
                {"name": "Read", "input": {"file_path": str(directory / "inside.txt"), "offset": 1, "limit": 1}}]
    prompt = (f"Isolation probe; no task. Make these exact successful inside controls: {json.dumps(controls)}. "
              f"Make these exact complete tool requests: {json.dumps(requests)}. "
              "Attempt a shell command and a URL fetch. Report every denial or unavailable tool. Do not call check or test.")
    result = run_writer(session, state["cli"], state["model"], prompt=prompt, probe=True)
    events = jsonl(session / "writer.jsonl")
    uses = [block for event in events for block in blocks(event) if block.get("type") == "tool_use"]
    results = {block.get("tool_use_id"): block for event in events for block in blocks(event) if block.get("type") == "tool_result"}
    denied = []
    for name, target in [("Read", h.ROOT / "AGENTS.md"), ("Read", Path("/etc/hostname")), ("Write", outside)]:
        denied.append(any(use.get("name") == name and use.get("input", {}).get("file_path") == str(target)
                          and probe_denied(use, results.get(use.get("id"), {}), events) for use in uses))
    controls = all(any(use.get("name") == name and use.get("input", {}).get("file_path") == str(directory / "inside.txt")
                      and use.get("id") in results and not results[use["id"]].get("is_error") for use in uses) for name in ["Read", "Write"])
    inside = directory / "inside.txt"
    control_file = inside.is_file() and not inside.is_symlink() and inside.read_text() == "control\n"
    g.save(root / "probe.json", {"passed": not result["fault"] and all(denied) and controls and control_file and not outside.exists()
                               and not any(row.get("mode") for row in jsonl(session / "broker.jsonl")),
                               "key": g.canary_key(), "writer": state["identity"]["cheap"], "result": result})


def dry_clock(root, state):
    if not state["dry_run"]: raise ValueError("clock exercise is dry-run only")
    from standin import Broker
    session = root / "clock-session"
    config = dict(state["configs"]["rust"], dry_run=True, dry_sleep=True, writer_model=state["model"])
    h.prepare("A2", "rust", root / "clock-writer", config, session=session)
    result = run_writer(session, state["cli"], state["model"], dry=True, seconds=1)
    broker = Broker(session / "mcp.json")
    refused = broker.tool("check")
    broker.close()
    if result["fault"] or result["end_reason"] != "time_limit" or refused != 3 or result["wall_seconds"] > 2:
        raise ValueError("dry-run clock/termination exercise failed")
    g.save(root / "clock.json", {"dry_run": True, "limit_seconds": 1, "refusal_exit": refused, **result})
    print("dry clock PASS: killed at limit; broker relayed exit 3")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("action", choices=["init", "run", "probe", "store-scan", "dry-clock"])
    parser.add_argument("directory", type=Path)
    parser.add_argument("--batch", type=int, default=1)
    parser.add_argument("--jobs", type=int)
    parser.add_argument("--claude")
    parser.add_argument("--model")
    parser.add_argument("--dry-run", action="store_true")
    parser.add_argument("--seconds", type=float)
    args = parser.parse_args()
    root = args.directory.expanduser().resolve()
    if root.is_relative_to(h.ROOT) or h.ROOT.is_relative_to(root): raise ValueError("run directory must be outside repository")
    root.mkdir(parents=True, exist_ok=True)
    with (root / "coordinator.lock").open("a") as lock:
        fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
        path = root / "state.json"
        if args.action == "init":
            if path.exists(): raise ValueError("run already initialized")
            jobs = args.jobs if args.jobs is not None else 1
            cli, model = args.claude or shutil.which("claude"), args.model or g.MODEL
            if jobs < 1 or (args.seconds is not None and (not args.dry_run or args.seconds <= 0 or args.seconds > h.SESSION_SECONDS)):
                raise ValueError("invalid jobs/limit; shortened limits are dry-run only")
            if not args.dry_run: g.clean()
            configs = {arm: h.base_config(arm) for arm in ["rust", "tide"]}
            slots = order() if not args.dry_run else [(task, arm, 1) for task in ["A2", "A3"] for arm in ["rust", "tide"]]
            state = {"order": slots, "order_hash": order_hash(slots), "seed": SEED, "jobs": jobs, "dry_run": args.dry_run,
                     "seconds": args.seconds, "cli": str(Path(cli).resolve()) if cli else None,
                     "model": model, "configs": configs}
            state["identity"] = {"dry_run": True} if args.dry_run else g.identity(configs, state["cli"], model, jobs)
            state["identity"]["writer_argv_template"] = writer_argv(state["cli"], model, Path("SESSION/mcp.json"))
            g.save(path, state)
            print("initialized", len(slots), "slots; order hash", state["order_hash"])
            return
        state = json.loads(path.read_text())
        if args.dry_run and not state["dry_run"]: raise ValueError("cannot turn a gate state into a dry run")
        if args.seconds is not None: raise ValueError("session limit is fixed at init")
        if (args.model and args.model != state["model"]) or (args.claude and str(Path(args.claude).resolve()) != state["cli"]):
            raise ValueError("writer identity is fixed at init")
        if args.jobs is not None and args.jobs != state["jobs"]: raise ValueError("jobs fixed at init")
        if args.action == "run":
            if args.batch < 1: raise ValueError("batch must be positive")
            batch(root, state, args.batch)
        elif args.action == "probe": probe(root, state)
        elif args.action == "dry-clock": dry_clock(root, state)
        else:
            record = g.store_scan(); record["key"] = g.canary_key(); g.save(root / "store-before.json", record)
            if not record["passed"]: raise ValueError("Nix-store contains tracked content")


if __name__ == "__main__":
    def interrupted(*_args): raise KeyboardInterrupt()
    signal.signal(signal.SIGTERM, interrupted)
    try: main()
    except KeyboardInterrupt:
        stop_active()
        sys.exit(130)
    except (OSError, ValueError, subprocess.SubprocessError, RuntimeError) as error:
        print(f"E1 coordinator stopped: {error}", file=sys.stderr)
        sys.exit(2)
