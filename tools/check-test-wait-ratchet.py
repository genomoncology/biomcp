#!/usr/bin/env python3
"""Report clock-based waits in tests without blocking acceptance.

Usage: check-test-wait-ratchet.py [--root DIR]

The existing scanner compares unmarked waits and watchdog markers with
stored ceilings. Findings are advisory. Inventory, enumeration and source
read errors fail the invocation. Historical raise ledgers and ticket review
wording do not admit waits, and this tool does not update the inventory.
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
# `\bsleep(` catches the bare form that follows a `use
# std::thread::sleep` import (and subsumes the qualified forms);
# `sleep_until(` parks a timer on the clock like any sleep; and ANY
# comparison against an `elapsed()` value on either side of the
# operator is a deadline poll (a `<` on the right, a chained
# `.as_millis()` on the left — the shapes the old `>` -only pattern
# missed). `>=` floor assertions are not comparisons against a
# deadline, so they stay out.
RUST_PATTERNS = [
    re.compile(pattern)
    for pattern in (
        # Only the SCALED HELPER call exempts a deadline addition; a
        # `watchdog(` word in a comment is not the helper (ticket
        # 1269).
        r"Instant::now\(\)\s*\+\s*(?!.*\b[\w:]*test_support::watchdog\()",
        r"\bsleep\s*\(",
        r"\bsleep_until\s*\(",
        # Any comparison against elapsed() in either operand order
        # and either operator is a deadline poll; the only exception
        # is a `>=` floor assertion (test took AT LEAST this long),
        # which the floor-exemption below subtracts.
        r"elapsed\(\)(?:\s*\.\s*\w+\(\))*\s*[<>]=?",
        r"[<>]=?\s*[\w.:]*elapsed\(",
        # A poll against the clock itself: `Instant::now() < deadline`
        # in either order (`+` additions are the allowed builder).
        r"Instant::now\(\)\s*[<>]=?",
        r"[<>]=?\s*[\w.:]*Instant::now\(\)",
        # `Instant::now().duration_since(s) < d` is the same poll in
        # its method form.
        r"duration_since\([^()]*\)\s*[<>]=?",
    )
]
PYTHON_PATTERNS = [
    re.compile(pattern)
    for pattern in (
        r"time\.sleep",
        r"asyncio\.sleep",
        r"anyio\.sleep",
        r"(?<![\w.])sleep\s*\(",
        r"from\s+time\s+import\s+[^\n]*\bsleep\b",
        # Monotonic-clock polls are waits like any sleep.
        r"time\.monotonic\(\)\s*[<>]=?",
        r"[<>]=?\s*[\w.]*time\.monotonic\(\)",
    )
]
# Aliased imports resolve to per-file local names:
# `import time as t` makes `t.sleep(` a timed wait, and
# `from time import sleep as snooze` makes `snooze(` one. The bare
# `from time import sleep` form is covered by its own pattern.
TIME_MODULE_ALIAS = re.compile(
    r"^\s*import\s+[\w\s,]*?\btime\s+as\s+(\w+)", re.MULTILINE
)
# Rust use-aliases: `use std::thread::sleep as nap` (and the tokio
# timer) make `nap(` a timed wait, resolved per file.
RUST_SLEEP_ALIAS = re.compile(
    r"use\s+(?:std::thread|tokio::time)::sleep\s+as\s+(\w+)"
    r"|use\s+(?:std::thread|tokio::time)::\{[^}]*?\bsleep\s+as\s+(\w+)\b[^}]*\}",
    re.MULTILINE,
)
# Stored clock values: `let e = start.elapsed();` then `if e < d` is
# a poll through a binding; so is a duration_since stored the same
# way. The comparison patterns gain the bound name per file.
RUST_TIME_BINDING = re.compile(
    r"let\s+(?:mut\s+)?(\w+)\s*=\s*[\w.:]*(?:elapsed\(\)"
    r"|Instant::now\(\)\s*\.\s*duration_since\([^)]*\))\s*(?:\.[\w.]+\(\))*\s*;",
)
# Python module-object assignment: `t = time` makes `t.sleep(` one.
PYTHON_TIME_ASSIGN = re.compile(r"^\s*(\w+)\s*=\s*time\s*$", re.MULTILINE)
SLEEP_RENAME_IMPORT = re.compile(
    r"^\s*from\s+time\s+import\s+sleep\s+as\s+(\w+)", re.MULTILINE
)
# An ASSERTION on a clock value bounds the test's own duration
# (took at least/at most this long); it does not park a timer, so it
# is subtracted from the comparison patterns above — either operator,
# either operand order.
FLOOR_ASSERTION = re.compile(
    r"assert[^;]*?(?:[\w.:]*(?:elapsed|Instant::now))\(\)"
    r"(?:\s*\.\s*\w+\(\))*\s*[<>]=?"
    r"|assert[^;]*?[<>]=?\s*[\w.:]*(?:elapsed|Instant::now)\("
    r"|assert[^;]*?duration_since\([^;]*?\)\s*[<>]=?"
)

# A `watchdog:` marker vouches for a wait only when it carries a
# reason: at least one word of three or more characters after the
# colon. A bare `watchdog:` (or `watchdog: x`) does not pass.
WATCHDOG_REASON = re.compile(r"watchdog:\s*\w{3,}")
DEFINITION = re.compile(
    r"^\s*(?:pub(?:\(.+?\))?\s+)?(?:async\s+)?fn\s+\w*heartbeat\w*|^\s*def\s+_?\w*heartbeat\w*",
    re.IGNORECASE,
)

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
        line for line in output.splitlines() if line and line.endswith(suffix)
    )


def rust_test_region(path: Path) -> str | None:
    """Whole file for test-only files; after the first #[cfg(test)] otherwise."""
    text = path.read_text(encoding="utf-8")
    posix = path.as_posix()
    if "/tests/" in posix or posix.startswith("tests/") or TEST_FILE_NAME.search(posix):
        return text
    marker = text.find("#[cfg(test)]")
    if marker == -1:
        return None
    return text[marker:]


def count_waits(
    lines: list[str],
    patterns: list[re.Pattern[str]],
    alias_patterns: list[re.Pattern[str]] | None = None,
) -> tuple[int, list[str], int]:
    """Unmarked waits, their line numbers, and the marked count.

    `alias_patterns` are the per-file local names bound to the time
    module or its sleep (from `import time as t` / `from time import
    sleep as s`): a call through any of them is a timed wait.
    """
    marked = 0
    violations: list[str] = []
    in_heartbeat_helper = False
    alias_patterns = alias_patterns or []
    assert_open = False
    for number, line in enumerate(lines, start=1):
        if DEFINITION.search(line):
            in_heartbeat_helper = True
        elif re.match(r"^\s*(?:pub(?:\(.+?\))?\s+)?(?:async\s+)?fn\s|^\s*def\s", line):
            in_heartbeat_helper = False
        waited = any(p.search(line) for p in patterns) or any(
            p.search(line) for p in alias_patterns
        )
        if waited and (
            FLOOR_ASSERTION.search(line) or (assert_open and re.search(r"[<>]=", line))
        ):
            waited = False
        # A multi-line assert! opens here: a following `elapsed >=`
        # line is its floor assertion.
        assert_open = bool(re.search(r"assert\w*[!(]?\s*\($", line))
        if "watchdog:" in line:
            if WATCHDOG_REASON.search(line) and waited:
                marked += 1
                continue
            # A marker without a reason text does not vouch for the
            # wait; fall through and count it like any other.
        if waited:
            if in_heartbeat_helper:
                violations.append(
                    f"{number} heartbeat helper contains a bare timed wait"
                )
            else:
                violations.append(f"{number}")
    return len(violations), violations, marked


def local_time_aliases(text: str) -> list[re.Pattern[str]]:
    aliases: list[str] = []
    for match in TIME_MODULE_ALIAS.finditer(text):
        aliases.append(re.escape(match.group(1)) + r"\.sleep\s*\(")
    for match in SLEEP_RENAME_IMPORT.finditer(text):
        aliases.append(r"(?<![\w.])" + re.escape(match.group(1)) + r"\s*\(")
    for match in PYTHON_TIME_ASSIGN.finditer(text):
        aliases.append(re.escape(match.group(1)) + r"\.sleep\s*\(")
    return [re.compile(a) for a in aliases]


def rust_sleep_aliases(text: str) -> list[re.Pattern[str]]:
    aliases: list[str] = []
    for match in RUST_SLEEP_ALIAS.finditer(text):
        name = next((g for g in match.groups() if g), None)
        if name:
            aliases.append(r"(?<![\w.])" + re.escape(name) + r"\s*\(")
    return [re.compile(a) for a in aliases]


def rust_time_bindings(text: str) -> list[re.Pattern[str]]:
    """Comparison patterns for stored clock values.

    `let e = start.elapsed();` then `if e < d` is a deadline poll
    through a name; a `>=` floor assertion stays exempt like the
    direct form.
    """
    patterns: list[str] = []
    for match in RUST_TIME_BINDING.finditer(text):
        name = re.escape(match.group(1))
        patterns.append(r"\b" + name + r"\b\s*[<>]=?")
        patterns.append(r"[<>]=?\s*\b" + name + r"\b")
    return [re.compile(a) for a in patterns]


def scan(root: Path) -> dict[str, dict[str, object]]:
    files: dict[str, dict[str, object]] = {}
    rust_files = tracked(root, "src", ".rs") + tracked(root, "tests", ".rs")
    for relative in rust_files:
        path = root / relative
        region = rust_test_region(path)
        if region is None:
            continue
        count, _, marked = count_waits(
            region.splitlines(),
            RUST_PATTERNS,
            rust_sleep_aliases(region) + rust_time_bindings(region),
        )
        if count or marked:
            entry: dict[str, object] = {"count": count, "language": "rust"}
            if marked:
                entry["markers"] = marked
            files[relative] = entry
    for relative in tracked(root, "tests", ".py"):
        path = root / relative
        text = path.read_text(encoding="utf-8")
        count, _, marked = count_waits(
            text.splitlines(), PYTHON_PATTERNS, local_time_aliases(text)
        )
        if count or marked:
            entry = {"count": count, "language": "python"}
            if marked:
                entry["markers"] = marked
            files[relative] = entry
    return files


def report(root: Path) -> int:
    inventory_path = root / "tools/test-wait-inventory.json"
    inventory = json.loads(inventory_path.read_text(encoding="utf-8"))
    if (
        not isinstance(inventory, dict)
        or inventory.get("schema") != "biomcp-test-wait-inventory-v1"
    ):
        raise ValueError("invalid test-wait inventory schema")
    pinned = inventory.get("files")
    if not isinstance(pinned, dict):
        raise ValueError("test-wait inventory files must be an object")
    pinned = {
        name: (entry if isinstance(entry, dict) else {"count": entry})
        for name, entry in pinned.items()
    }
    for name, entry in pinned.items():
        for field in ("count", "markers"):
            value = entry.get(field, 0)
            if type(value) is not int or value < 0:
                raise ValueError(f"invalid test-wait {field} ceiling: {name}")
    current = scan(root)
    total_markers = sum(int(e.get("markers", 0)) for e in current.values())
    marker_ceiling = inventory.get("marker_total_ceiling", total_markers)
    if type(marker_ceiling) is not int or marker_ceiling < 0:
        raise ValueError("invalid test-wait marker total ceiling")

    warnings: list[str] = []
    notes: list[str] = []
    for name, entry in sorted(current.items()):
        count = entry["count"]
        ceiling = pinned.get(name)
        if ceiling is None:
            warnings.append(
                f"{name} is not in the inventory ({count} unmarked waits, "
                f"{entry.get('markers', 0)} watchdog markers)"
            )
            continue
        if count > ceiling.get("count", 0):
            warnings.append(
                f"{name} has {count} unmarked waits, above the pinned ceiling "
                f"{ceiling.get('count', 0)}"
            )
        elif count < ceiling.get("count", 0):
            notes.append(f"{name} dropped {ceiling['count']} -> {count}")
        file_markers = entry.get("markers", 0)
        pinned_markers = ceiling.get("markers", 0)
        if file_markers > pinned_markers:
            warnings.append(
                f"{name} carries {file_markers} watchdog markers, above its "
                f"pinned {pinned_markers}"
            )
    for name in sorted(set(pinned) - set(current)):
        notes.append(f"{name} now has zero waits and watchdog markers")
    if total_markers > marker_ceiling:
        warnings.append(
            f"the tree carries {total_markers} watchdog markers, above the global ceiling {marker_ceiling}"
        )
    for note in notes:
        print(f"note: {note}")
    for warning in warnings:
        print(f"warning: {warning}")
    print(
        f"test-wait scan advisory ({len(current)} files, "
        f"{total_markers} watchdog markers, {len(warnings)} warnings)"
    )
    return 0


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--root", type=Path, default=Path(__file__).resolve().parents[1]
    )
    args = parser.parse_args(argv)
    try:
        return report(args.root)
    except (OSError, UnicodeError, ValueError, subprocess.SubprocessError) as error:
        print(f"test-wait scan error: {error}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
