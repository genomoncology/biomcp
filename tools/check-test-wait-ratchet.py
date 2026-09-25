#!/usr/bin/env python3
"""Ceiling ratchet against new clock-based waits in tests.

Usage: check-test-wait-ratchet.py [--update] [--root DIR]

Waits should be signals (a handshake line, an end-of-input, a kernel
state) — see ticket 1252. This ratchet scans test code for the clock
shapes that made the suite flaky under load:

  Rust   Instant::now() + ..., thread::sleep, tokio::time::sleep
  Python time.sleep

plus test helpers named *heartbeat* that contain a bare sleep. A line
carrying a `watchdog: <reason>` comment passes: it declares a bounded
watchdog that a human vouched for.

Every scanned file is pinned in tools/test-wait-inventory.json as a
per-file ceiling. Counts above the ceiling fail. Counts below it pass;
re-pin them down deliberately with `--update` (the same discipline as
tools/update-rust-source-size-inventory) so decreases are reviewed,
not automatic.
"""

from __future__ import annotations

import argparse
import json
import re
import subprocess
import sys
from pathlib import Path

# `Instant::now() + watchdog(..)` routes through the shared scaled
# builder and does not count; every other deadline addition does.
RUST_PATTERNS = [
    re.compile(pattern)
    for pattern in (
        r"Instant::now\(\)\s*\+\s*(?!.*\bwatchdog\()",
        r"thread::sleep",
        r"tokio::time::sleep",
    )
]
PYTHON_PATTERNS = [re.compile(r"time\.sleep")]
DEFINITION = re.compile(r"^\s*(?:pub(?:\(.+?\))?\s+)?(?:async\s+)?fn\s+\w*heartbeat\w*|^\s*def\s+_?\w*heartbeat\w*", re.IGNORECASE)

TEST_FILE_NAME = re.compile(r"(?:^|/)(?:tests?\.rs|test_support\.rs)$")


def tracked(root: Path, directory: str, suffix: str) -> list[str]:
    # `git ls-files -- <dir>` plus an in-process suffix filter: git's
    # `**` pathspec magic varies by version, so the glob stays ours.
    output = subprocess.run(
        ["git", "ls-files", "--", directory],
        cwd=root,
        check=True,
        capture_output=True,
        text=True,
    ).stdout
    return sorted(
        line
        for line in output.splitlines()
        if line and line.endswith(suffix)
    )


def rust_test_region(path: Path) -> str | None:
    """Whole file for test-only files; after the first #[cfg(test)] otherwise."""
    text = path.read_text(encoding="utf-8")
    if "/tests/" in path.as_posix() or TEST_FILE_NAME.search(path.as_posix()):
        return text
    marker = text.find("#[cfg(test)]")
    if marker == -1:
        return None
    return text[marker:]


def count_waits(lines: list[str], patterns: list[re.Pattern[str]]) -> tuple[int, list[str]]:
    marked = 0
    violations: list[str] = []
    in_heartbeat_helper = False
    for number, line in enumerate(lines, start=1):
        if DEFINITION.search(line):
            in_heartbeat_helper = True
        elif re.match(r"^\s*(?:pub(?:\(.+?\))?\s+)?(?:async\s+)?fn\s|^\s*def\s", line):
            in_heartbeat_helper = False
        if "watchdog:" in line:
            if any(p.search(line) for p in patterns):
                marked += 1
            continue
        if any(p.search(line) for p in patterns):
            if in_heartbeat_helper:
                violations.append(
                    f"{number} heartbeat helper contains a bare timed wait"
                )
            else:
                violations.append(f"{number}")
    return len(violations), violations


def scan(root: Path) -> dict[str, dict[str, object]]:
    files: dict[str, dict[str, object]] = {}
    rust_files = tracked(root, "src", ".rs") + tracked(root, "tests", ".rs")
    for relative in rust_files:
        path = root / relative
        region = rust_test_region(path)
        if region is None:
            continue
        count, _ = count_waits(region.splitlines(), RUST_PATTERNS)
        if count:
            files[relative] = {"count": count, "language": "rust"}
    for relative in tracked(root, "tests", ".py"):
        path = root / relative
        count, _ = count_waits(
            path.read_text(encoding="utf-8").splitlines(), PYTHON_PATTERNS
        )
        if count:
            files[relative] = {"count": count, "language": "python"}
    return files


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parents[1])
    parser.add_argument(
        "--update",
        action="store_true",
        help="re-pin every ceiling to the current counts (use to ratchet down)",
    )
    args = parser.parse_args()
    inventory_path = args.root / "tools/test-wait-inventory.json"
    inventory = json.loads(inventory_path.read_text(encoding="utf-8"))
    pinned: dict[str, int] = {
        name: entry["count"] if isinstance(entry, dict) else entry
        for name, entry in inventory.get("files", {}).items()
    }
    current = scan(args.root)

    failures: list[str] = []
    notes: list[str] = []
    for name, count in sorted((n, e["count"]) for n, e in current.items()):
        ceiling = pinned.get(name)
        if ceiling is None:
            failures.append(f"new unmarked timed waits in {name} ({count}); convert to a signal or mark each with `watchdog:`")
        elif count > ceiling:
            failures.append(f"{name} has {count} unmarked waits, above the pinned ceiling {ceiling}")
        elif count < ceiling:
            notes.append(f"{name} dropped {ceiling} -> {count}; re-pin down with --update")
    for name in sorted(set(pinned) - set(current)):
        notes.append(f"{name} now has zero unmarked waits; re-pin down with --update")

    if args.update:
        inventory["files"] = {name: entry for name, entry in sorted(current.items())}
        inventory_path.write_text(json.dumps(inventory, indent=2) + "\n", encoding="utf-8")
        print(f"re-pinned {len(current)} file ceilings in {inventory_path}")
        return 0

    for note in notes:
        print(f"note: {note}")
    if failures:
        print("test-wait ratchet failures:")
        for failure in failures:
            print(f"  {failure}")
        return 1
    print(f"test-wait ratchet ok ({len(current)} files with pinned ceilings)")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
