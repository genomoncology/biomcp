#!/usr/bin/env python3
"""Check declared feature links cheaply; report separately collected LLVM coverage."""
from __future__ import annotations

import argparse
import json
import subprocess
import sys
from pathlib import Path

from feature_coverage import coverage_report, inventory


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parents[1])
    parser.add_argument("mode", choices=("links", "report", "python-tests", "rust-targets"))
    parser.add_argument("--lcov", type=Path)
    parser.add_argument("--statuses", type=Path)
    parser.add_argument("--floor", type=float)
    args = parser.parse_args()
    try:
        if args.mode == "report":
            if args.floor is None or not 0 <= args.floor <= 100:
                raise ValueError("coverage floor must be configured between 0 and 100")
            result = coverage_report(args.root, args.lcov,
                                     json.loads(args.statuses.read_text()), args.floor)
            result["source"] = subprocess.check_output(
                ["git", "rev-parse", "HEAD"], cwd=args.root, text=True).strip() if (args.root / ".git").exists() else "fixture"
            print(json.dumps(result, indent=2))
            return int(result["status"] != "pass")
        links, config, errors = inventory(args.root)
        if errors:
            print("\n".join(errors), file=sys.stderr)
            return 1
        if args.mode == "python-tests":
            for test in sorted({link["test"].split("[")[0] for link in links if link["kind"] == "python"}):
                print(test)
        elif args.mode == "rust-targets":
            for path in sorted(config["rust"]):
                print(Path(path).stem)
        else:
            print(f"Feature links pass: {len(links)} outside-in declarations")
        return 0
    except (OSError, ValueError, KeyError, StopIteration, SyntaxError) as error:
        print(f"feature coverage: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
