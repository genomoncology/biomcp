#!/usr/bin/env python3
"""Add host load to nextest's nonterminating slow-test reports."""

from __future__ import annotations

import os
import re
import signal
import subprocess
import sys
from pathlib import Path


def main() -> int:
    config = (Path(__file__).resolve().parents[1] / ".config/nextest.toml").read_text()
    default_profile = config.partition("[profile.default]")[2].partition("[profile.")[0]
    warning_period = re.search(r'period\s*=\s*"([0-9.]+)s"', default_profile)
    assert warning_period is not None, "configure the routine warning period in seconds"
    threshold = float(warning_period.group(1))
    warned: set[str] = set()
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
            timing = re.match(r"\s*(?:SLOW|PASS|FAIL|TIMEOUT)\s+\[\s*>?\s*([0-9.]+)s\]\s+(.*)", line)
            identity = re.sub(r"^\([^)]*\)\s+", "", timing.group(2)) if timing else ""
            if timing and (float(timing.group(1)) > threshold or line.lstrip().startswith("SLOW")) and identity not in warned:
                warned.add(identity)
                try:
                    load = f"{os.getloadavg()[0]:.2f}"
                except (AttributeError, OSError):
                    load = "unavailable"
                print(
                    f"WARNING slow test {identity} ({timing.group(1)}s); load (1 min): {load}",
                    flush=True,
                )
        status = process.wait()
        return status if status >= 0 else 128 - status


if __name__ == "__main__":
    raise SystemExit(main())
