#!/usr/bin/env python3
"""Add host load to nextest's nonterminating slow-test reports."""
from __future__ import annotations

import os
import re
import subprocess
import sys


def main() -> int:
    with subprocess.Popen(
        sys.argv[1:], stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True
    ) as process:
        assert process.stdout is not None
        for line in process.stdout:
            print(line, end="", flush=True)
            if re.match(r"\s*SLOW\s+\[", line):
                load = ", ".join(f"{value:.2f}" for value in os.getloadavg())
                print(f"WARNING slow test; load (1, 5, 15 min): {load}", flush=True)
        return process.wait()


if __name__ == "__main__":
    raise SystemExit(main())
