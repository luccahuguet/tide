#!/usr/bin/env python3
"""Linux parent-death guard for the entire writer process group."""
import ctypes
import json
import os
from pathlib import Path
import signal
import subprocess
import sys
import time


def main():
    parent, grant, session = int(sys.argv[1]), int(sys.argv[2]), Path(sys.argv[3])
    def stop(*_args):
        os.killpg(os.getpgrp(), signal.SIGKILL)
    signal.signal(signal.SIGTERM, stop)
    if ctypes.CDLL(None, use_errno=True).prctl(1, signal.SIGTERM, 0, 0, 0):
        raise OSError(ctypes.get_errno(), "cannot install writer parent-death guard")
    if os.getppid() != parent: stop()
    # No writer can start until its coordinator has persisted the group identity.
    if os.read(grant, 1) != b"g": stop()
    os.close(grant)
    child = subprocess.Popen(sys.argv[4:])
    code = child.wait()
    path = session / "process-exit.json"
    tmp = path.with_suffix(".tmp")
    tmp.write_text(json.dumps({"exit": code}) + "\n")
    tmp.replace(path)
    until = time.monotonic() + 3
    while time.monotonic() < until:
        live = False
        for path in Path("/proc").iterdir():
            if not path.name.isdigit() or int(path.name) == os.getpid(): continue
            try: fields = (path / "stat").read_text().rsplit(")", 1)[1].split()
            except (FileNotFoundError, ProcessLookupError): continue
            if int(fields[2]) == os.getpgrp() and fields[0] != "Z":
                live = True; break
        if not live: break
        time.sleep(0.02)
    stop()


if __name__ == "__main__": main()
