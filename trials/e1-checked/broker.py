#!/usr/bin/env python3
"""Owned, argument-free E1 check/test MCP relay (newline JSON-RPC on stdio)."""
import json
import os
from pathlib import Path
import signal
import subprocess
import sys
import time

CAP = 1_000_000
HERE = Path(__file__).resolve().parent


def serve(session):
    for line in sys.stdin:
        request = json.loads(line)
        if "id" not in request:
            continue
        method, params = request.get("method"), request.get("params", {})
        reply = {"jsonrpc": "2.0", "id": request["id"]}
        if method == "initialize":
            reply["result"] = {"protocolVersion": params["protocolVersion"], "capabilities": {"tools": {}},
                               "serverInfo": {"name": "e1", "version": "1"}}
        elif method == "ping":
            reply["result"] = {}
        elif method == "tools/list":
            reply["result"] = {"tools": [{"name": mode, "description": f"Runs ./{mode} as your instructions describe.",
                "inputSchema": {"type": "object", "properties": {}, "additionalProperties": False}}
                for mode in ["check", "test"]]}
        elif method == "tools/call" and params.get("name") in {"check", "test"} and params.get("arguments", {}) == {}:
            mode, started = params["name"], time.monotonic()
            proc = subprocess.Popen([sys.executable, str(HERE / "harness.py"), "command", str(session), mode],
                stdin=subprocess.DEVNULL, stdout=subprocess.PIPE, stderr=subprocess.PIPE, start_new_session=True)
            try:
                out, err = proc.communicate(timeout=150)
                code = proc.returncode
            except subprocess.TimeoutExpired:
                os.killpg(proc.pid, signal.SIGKILL)
                out, err = proc.communicate()
                code = 2
                err += b"\nbroker guard timeout\n"
            raw = out + err
            capped = len(raw) > CAP
            message = raw[:CAP].decode(errors="replace")
            if capped:
                message += "\n[E1 output truncated]\n"
            message += f"\nexit: {code}\n"
            completed = json.loads((session / "session.json").read_text())["runs"] if code != 3 else None
            with (session / "broker.jsonl").open("a") as log:
                log.write(json.dumps({"time": time.time(), "mode": mode, "exit": code,
                    "seconds": time.monotonic() - started, "truncated": capped, "bytes": len(raw), "completed_run": completed}) + "\n")
            reply["result"] = {"content": [{"type": "text", "text": message}], "isError": code != 0}
        else:
            reply["error"] = {"code": -32602, "message": "Only argument-free check and test are allowed"}
        print(json.dumps(reply), flush=True)


if __name__ == "__main__":
    session = Path(sys.argv[1]).resolve()
    def lifecycle(event):
        with (session / "broker.jsonl").open("a") as log:
            log.write(json.dumps({"event": event, "pid": os.getpid(), "time": time.time()}) + "\n")
    signal.signal(signal.SIGTERM, lambda *_: sys.exit(0))
    lifecycle("start")
    try:
        serve(session)
    except Exception as error:
        with (session / "broker.jsonl").open("a") as log:
            log.write(json.dumps({"fault": "broker_crash", "error": str(error)}) + "\n")
        raise
    finally:
        lifecycle("stop")
