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
            FLOOR_ASSERTION.search(line)
            or (assert_open and re.search(r"[<>]=", line))
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


def raise_is_accepted(root: Path, record: dict) -> tuple[bool, str]:
    """A ceiling raise counts only with a reason and an ACCEPTED review.

    The record names the ticket whose Review accepted the raise; the
    ratchet reads that ticket file and requires an accepted code
    review. A raise citing the ticket currently in review fails the
    gate until the reviewer accepts — that is the discipline: no
    unreviewed raise reaches a green gate.
    """
    reason = str(record.get("reason", "")).strip()
    ticket = str(record.get("ticket", "")).strip()
    if not reason or not ticket:
        return False, "raise record needs a reason and a ticket"
    matches = sorted((root / "sdlc" / "tickets").glob(f"{ticket}-*.md"))
    if not matches:
        return False, f"raise cites ticket {ticket}, which has no file"
    text = matches[0].read_text(encoding="utf-8")
    verdicts = re.finditer(
        r"^\s*-?\s*\**code\s+re(?:view|-review)\**\s*(?:\([^)]*\))?\s*:"
        r"(?P<rest>.{0,200})",
        text,
        re.IGNORECASE | re.MULTILINE,
    )
    # A raise counts only when a verdict line's VALUE starts with
    # ACCEPT (2026-09-30 go-request review): promises, expectations
    # and pendings that merely mention ACCEPT — "awaiting ACCEPT",
    # "ACCEPT expected after fixes", "ACCEPT once fixes land",
    # "reviewer returns ACCEPT or findings", "REJECT, not ACCEPT yet"
    # — stay rejections. A history line like "REJECT once, folded;
    # ACCEPT 2026-09-28 ..." qualifies because its value continues
    # from ACCEPT after the semicolon only if the ACCEPT token starts
    # the value; folded history therefore records its acceptance on
    # its own verdict line.
    for verdict in verdicts:
        rest = verdict.group("rest").strip()
        # ACCEPT must be the verdict itself: the token starts the
        # value AND is not immediately an expectation ("ACCEPT
        # expected after fixes", "ACCEPT once fixes land").
        # ACCEPT counts only when followed by nothing, a date, or a
        # reviewer/dispatch reference (third go-request review):
        # "ACCEPT - will fix", "ACCEPT (pending)", "ACCEPT; will fix
        # later" and "ACCEPT, to be confirmed" stay rejections.
        if re.match(
            r"ACCEPT(?=[\s]*(?:$|\d{4}-\d{2}-\d{2}|by |from |dispatch|#[0-9a-f]))",
            rest,
        ):
            return True, ""
    return False, (
        f"raise cites ticket {ticket}, whose code review has not "
        f"been accepted — the gate stays red until it is"
    )


def accepted_raise_for(
    raises: list[dict], name: str, field: str, value: int, root: Path
) -> tuple[bool, str]:
    for record in raises:
        if (
            record.get("file") == name
            and record.get("field") == field
            and int(record.get("to", -1)) == value
        ):
            ok, why = raise_is_accepted(root, record)
            if ok:
                return True, ""
            return False, why
    return False, (
        f"no accepted raise record for {name} {field} -> {value}; add one "
        f"with a reason and the ticket whose review accepted it"
    )


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--root", type=Path, default=Path(__file__).resolve().parents[1]
    )
    parser.add_argument(
        "--update",
        action="store_true",
        help="re-pin every ceiling to the current counts (use to ratchet down)",
    )
    args = parser.parse_args()
    inventory_path = args.root / "tools/test-wait-inventory.json"
    inventory = json.loads(inventory_path.read_text(encoding="utf-8"))
    raises = list(inventory.get("raises", []))
    pinned: dict[str, dict] = {
        name: (entry if isinstance(entry, dict) else {"count": entry})
        for name, entry in inventory.get("files", {}).items()
    }
    current = scan(args.root)

    failures: list[str] = []
    notes: list[str] = []
    total_markers = sum(int(e.get("markers", 0)) for e in current.values())
    marker_ceiling = int(inventory.get("marker_total_ceiling", total_markers))
    for name, entry in sorted(current.items()):
        count = entry["count"]
        ceiling = pinned.get(name)
        if ceiling is None and count:
            failures.append(
                f"new unmarked timed waits in {name} ({count}); convert to a signal or mark each with `watchdog:`"
            )
        elif ceiling is None:
            # Markers only (count 0): pin the file so its marker count
            # is ratcheted too.
            failures.append(
                f"{name} carries `watchdog:` markers but is not in the inventory; "
                f"run with --update to pin it"
            )
        elif count > ceiling.get("count", count):
            ok, why = accepted_raise_for(
                raises, name, "count", count, args.root
            )
            if not ok:
                failures.append(
                    f"{name} has {count} unmarked waits, above the pinned ceiling "
                    f"{ceiling.get('count')} ({why})"
                )
        elif count < ceiling.get("count", count):
            notes.append(
                f"{name} dropped {ceiling.get('count')} -> {count}; re-pin down with --update"
            )
        pinned_markers = int(ceiling.get("markers", 0)) if ceiling else 0
        file_markers = int(entry.get("markers", 0))
        if ceiling is not None and file_markers > pinned_markers:
            ok, why = accepted_raise_for(
                raises, name, "markers", file_markers, args.root
            )
            if not ok:
                failures.append(
                    f"{name} carries {file_markers} `watchdog:` markers, above its "
                    f"pinned {pinned_markers} ({why})"
                )
    for name in sorted(set(pinned) - set(current)):
        notes.append(f"{name} now has zero unmarked waits; re-pin down with --update")
    # The raises list is the append-only history of every raise; a
    # pin that disagrees with its head is a silent edit bypassing
    # review (the tool cannot see yesterday's value any other way).
    by_key: dict[tuple[str, str], list[dict]] = {}
    for record in raises:
        by_key.setdefault((str(record.get("file")), str(record.get("field"))), []).append(record)
    for (name, field), records in by_key.items():
        head = records[-1]
        target = int(head.get("to", -1))
        if field == "marker_total_ceiling":
            actual = marker_ceiling
        else:
            entry = pinned.get(name) or {}
            actual = int(entry.get(field, -1)) if isinstance(entry, dict) else -1
        if actual != target:
            failures.append(
                f"{name} pin for {field} is {actual} but the last accepted raise "
                f"record says {target}; the raises list is the only history — "
                f"either restore the pin or append a reviewed raise"
            )
        steps = [(int(r.get("from", -1)), int(r.get("to", -1))) for r in records]
        for (earlier_from, earlier_to), (later_from, _) in zip(steps, steps[1:]):
            if earlier_to != later_from:
                failures.append(
                    f"{name} {field} raise chain is discontinuous: {steps}"
                )
        # Every raise in the chain must cite a ticket whose review
        # carries an ACCEPT — a folded raise (pin moved and record
        # appended in one commit) is checked here, not only on the
        # over-pin failure path, so no unreviewed raise can sit
        # green under a matching pin.
        for record in records:
            ok, why = raise_is_accepted(args.root, record)
            if not ok:
                failures.append(
                    f"{name} {field} raise {record.get('from')}->"
                    f"{record.get('to')} lacks an accepted review ({why})"
                )
    if total_markers > marker_ceiling:
        ok, why = accepted_raise_for(
            raises, "(global)", "marker_total_ceiling", total_markers, args.root
        )
        if not ok:
            failures.append(
                f"the tree carries {total_markers} `watchdog:` markers, above the "
                f"global ceiling {marker_ceiling} ({why})"
            )

    for note in notes:
        print(f"note: {note}")
    if failures:
        print("test-wait ratchet failures:")
        for failure in failures:
            print(f"  {failure}")
        return 1

    if args.update:
        # --update ratchets DOWN or holds; it must never silently
        # absorb a raise (2026-09-29 review). Any file whose current
        # count or marker total exceeds its pin needs an accepted
        # raise record first — the comparison loop above already
        # failed the run in that case, so reaching here means every
        # change is a decrease or a hold.
        fresh = {name: entry for name, entry in sorted(current.items())}
        lowered = sum(
            1
            for name, entry in fresh.items()
            if name in pinned
            and int(entry["count"]) < int(pinned[name].get("count", entry["count"]))
        )
        inventory["files"] = fresh
        inventory["marker_total_ceiling"] = sum(
            int(e.get("markers", 0)) for e in fresh.values()
        )
        inventory_path.write_text(
            json.dumps(inventory, indent=2) + "\n", encoding="utf-8"
        )
        print(
            f"re-pinned {len(fresh)} file ceilings in {inventory_path} "
            f"({lowered} lowered; no raise absorbed — raises require an "
            f"accepted review record)"
        )
        return 0
    print(
        f"test-wait ratchet ok ({len(current)} files with pinned ceilings, "
        f"{total_markers} watchdog markers under the ceiling {marker_ceiling})"
    )
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
