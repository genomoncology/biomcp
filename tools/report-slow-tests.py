#!/usr/bin/env python3
"""Add host load to nextest's nonterminating slow-test reports."""

from __future__ import annotations

import os
import re
import signal
import subprocess
import sys


def main() -> int:
    with subprocess.Popen(
        sys.argv[1:], stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True
    ) as process:

        def forward_signal(signum: int, _frame: object) -> None:
            if process.poll() is None:
                process.send_signal(signum)

        for signum in (signal.SIGINT, signal.SIGTERM, signal.SIGHUP):
            signal.signal(signum, forward_signal)
        assert process.stdout is not None
        for line in process.stdout:
            print(line, end="", flush=True)
            if re.match(r"\s*SLOW\s+\[", line):
                try:
                    load = ", ".join(f"{value:.2f}" for value in os.getloadavg())
                except (AttributeError, OSError):
                    load = "unavailable"
                print(f"WARNING slow test; load (1, 5, 15 min): {load}", flush=True)
        status = process.wait()
        return status if status >= 0 else 128 - status


if __name__ == "__main__":
    raise SystemExit(main())
