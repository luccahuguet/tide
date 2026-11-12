#!/usr/bin/env python3
"""Task-blind dry-run writer. All check/test calls use the real MCP broker."""
import json
from pathlib import Path
import subprocess
import sys
import time

import gate_checks as g


def emit(event):
    print(json.dumps(event), flush=True)


class Broker:
    def __init__(self, mcp):
        spec = json.loads(Path(mcp).read_text())["mcpServers"]["e1"]
        self.proc = subprocess.Popen([spec["command"], *spec["args"]], stdin=subprocess.PIPE,
                                     stdout=subprocess.PIPE, text=True)
        self.number = 0
        self.call("initialize", {"protocolVersion": "2024-11-05", "capabilities": {}, "clientInfo": {"name": "dry", "version": "1"}})
        tools = self.call("tools/list", {})["result"]["tools"]
        assert {tool["name"] for tool in tools} == {"check", "test"}

    def call(self, method, params):
        self.number += 1
        request = {"jsonrpc": "2.0", "id": self.number, "method": method, "params": params}
        self.proc.stdin.write(json.dumps(request) + "\n"); self.proc.stdin.flush()
        reply = json.loads(self.proc.stdout.readline())
        assert reply["id"] == self.number
        return reply

    def tool(self, mode):
        reply = self.call("tools/call", {"name": mode, "arguments": {}})
        return int(reply["result"]["content"][0]["text"].rsplit("exit: ", 1)[1])

    def close(self):
        self.proc.stdin.close()
        assert self.proc.wait(timeout=10) == 0
        self.proc.stdout.close()


def main():
    session, mcp = map(Path, sys.argv[1:])
    config = json.loads((session / "session.json").read_text())
    emit({"type": "system", "subtype": "init", "tools": sorted(g.TOOLS), "plugins": [],
          "mcp_servers": [{"name": "e1", "status": "connected"}], "model": config.get("writer_model", g.MODEL)})
    if config.get("dry_sleep"):
        time.sleep(60)
        return
    broker = Broker(mcp)
    entry = next((Path(config["directory"]) / "src").glob("*.rs"))
    entry.write_text('compile_error!("dry-run control");\n')
    assert broker.tool("check") == 1
    # Deliberately unrelated source: no held-out program or task content is used.
    entry.write_text("pub fn e1_dry_control() {}\n")
    assert broker.tool("test") in (0, 1)
    for _ in range(18): assert broker.tool("check") in (0, 1)
    time.sleep(60)  # The coordinator must stop this writer at exhaustion, before it can edit again.
    entry.write_text("compile_error!(\"late edit\");\n")


if __name__ == "__main__": main()
