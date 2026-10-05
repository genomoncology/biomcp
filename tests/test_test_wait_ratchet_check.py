"""Contracts for advisory test waits and blocking scanner errors."""

from __future__ import annotations

import importlib.util
import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SPEC = importlib.util.spec_from_file_location(
    "check_test_wait_ratchet", ROOT / "tools" / "check-test-wait-ratchet.py"
)
MODULE = importlib.util.module_from_spec(SPEC)
assert SPEC.loader is not None
SPEC.loader.exec_module(MODULE)
scan = MODULE.scan
count_waits = MODULE.count_waits

INVENTORY = ROOT / "tools" / "test-wait-inventory.json"


def _run(root: Path, *args: str) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        [
            sys.executable,
            str(ROOT / "tools" / "check-test-wait-ratchet.py"),
            "--root",
            str(root),
            *args,
        ],
        capture_output=True,
        text=True,
        check=False,
        cwd=ROOT,
    )


def test_inventory_counts_match_the_tree() -> None:
    """The real tree passes against its pinned ceilings."""
    result = _run(ROOT)
    assert result.returncode == 0, result.stdout + result.stderr


def test_a_new_unmarked_wait_warns(tmp_path: Path) -> None:
    """An unmarked time.sleep warns without failing acceptance."""  # watchdog: synthetic literal
    root = tmp_path / "repo"
    (root / "tools").mkdir(parents=True)
    (root / "tools" / "test-wait-inventory.json").write_text(
        json.dumps({"schema": "biomcp-test-wait-inventory-v1", "files": {}}),
        encoding="utf-8",
    )
    tests_dir = root / "tests"
    tests_dir.mkdir()
    (tests_dir / "test_example.py").write_text(
        "def test_waits() -> None:\n    time.sleep(0.1)\n",  # watchdog: synthetic literal
        encoding="utf-8",
    )
    # scan() shells out to git ls-files, so stage the file.
    subprocess.run(["git", "init", "-q"], cwd=root, check=True)
    subprocess.run(["git", "add", "."], cwd=root, check=True)

    result = _run(root)
    assert result.returncode == 0
    assert (
        "warning: tests/test_example.py is not in the inventory (1 unmarked waits"
        in result.stdout
    )


def test_the_new_poll_and_alias_shapes_count() -> None:
    """The 2026-09-28 review shapes: each must count as a wait."""
    RUST_PATTERNS = MODULE.RUST_PATTERNS
    rust_sleep_aliases = MODULE.rust_sleep_aliases

    rust_lines = [
        "    while start.elapsed() <= limit { poll(); }",
        "    while limit >= start.elapsed() { poll(); }",
        '    assert!(Instant::now() < deadline, "readiness");',
        "    while deadline > Instant::now() { poll(); }",
    ]
    for line in rust_lines:
        assert any(pat.search(line) for pat in RUST_PATTERNS), line

    aliased = "use std::thread::sleep as nap;\n    nap(2);"
    assert any(pat.search("    nap(2);") for pat in rust_sleep_aliases(aliased)), (
        "aliased Rust sleep must resolve"
    )

    local_time_aliases = MODULE.local_time_aliases

    python_alias = "t = time\n    t.sleep(1)"
    assert any(
        pat.search("    t.sleep(1)") for pat in local_time_aliases(python_alias)
    ), "assigned Python time module must resolve"


def test_a_watchdog_marker_passes() -> None:
    lines = ["    time.sleep(0.05)  # watchdog: shared poll interval inside the helper"]
    count, violations, _ = count_waits(lines, [MODULE.PYTHON_PATTERNS[0]])
    assert count == 0
    assert violations == []


def test_a_heartbeat_helper_with_a_bare_sleep_counts() -> None:
    lines = [
        "def _heartbeat_advances(path):",
        "    before = path.read_text()",
        "    time.sleep(0.2)",  # watchdog: synthetic literal
        "    return path.read_text() != before",
    ]
    count, violations, _ = count_waits(lines, [MODULE.PYTHON_PATTERNS[0]])
    assert count == 1
    assert violations == ["3 heartbeat helper contains a bare timed wait"]


def test_watchdog_builder_deadlines_do_not_count() -> None:
    lines = [
        "    let deadline = std::time::Instant::now() + crate::test_support::watchdog(30);",
        "    let deadline = tokio::time::Instant::now() + test_support::watchdog(60);",
        "    let deadline = tokio::time::Instant::now() + Duration::from_secs(60);",  # watchdog: synthetic literal
        # The word watchdog( in a comment is not the scaled helper
        # and does not exempt the line; with no `watchdog:` marker
        # the line counts as an unmarked wait (ticket 1269).
        "    let d2 = tokio::time::Instant::now() + Duration::from_secs(5); // mirrors watchdog(60)",
    ]
    count, violations, _ = count_waits(lines, MODULE.RUST_PATTERNS)
    assert count == 2
    assert violations == ["3", "4"]


def test_an_inventory_decrease_passes_and_notes_the_ratchet_down(
    tmp_path: Path,
) -> None:
    """A count below the pinned ceiling passes; the note says re-pin down."""
    root = tmp_path / "repo"
    (root / "tools").mkdir(parents=True)
    (root / "tools" / "test-wait-inventory.json").write_text(
        json.dumps(
            {
                "schema": "biomcp-test-wait-inventory-v1",
                "files": {"tests/test_example.py": {"count": 2, "language": "python"}},
            }
        ),
        encoding="utf-8",
    )
    tests_dir = root / "tests"
    tests_dir.mkdir()
    (tests_dir / "test_example.py").write_text(
        "def test_waits() -> None:\n    time.sleep(0.1)\n",  # watchdog: synthetic literal
        encoding="utf-8",
    )
    subprocess.run(["git", "init", "-q"], cwd=root, check=True)
    subprocess.run(["git", "add", "."], cwd=root, check=True)

    result = _run(root)
    assert result.returncode == 0, result.stdout + result.stderr
    assert "dropped 2 -> 1" in result.stdout
    assert "warning:" not in result.stdout


def test_a_bare_sleep_after_the_time_import_counts() -> None:
    """The time-module sleep import plus a bare call is a timed wait."""  # watchdog: synthetic literal
    lines = [
        "from time import sleep",  # watchdog: synthetic literal
        "def test_waits() -> None:",
        "    sleep(0.1)",  # watchdog: synthetic literal
    ]
    count, _, _ = count_waits(lines, MODULE.PYTHON_PATTERNS)
    assert count == 2  # the import line and the call


def test_asyncio_sleep_counts() -> None:
    lines = ["    await asyncio.sleep(0.2)"]  # watchdog: synthetic literal
    count, _, _ = count_waits(lines, MODULE.PYTHON_PATTERNS)
    assert count == 1


def test_a_bare_rust_sleep_after_a_use_import_counts() -> None:
    """A use-import of sleep followed by a bare call is a timed wait."""  # watchdog: synthetic literal
    lines = [
        "    use std::thread::sleep;",  # watchdog: synthetic literal
        "    sleep(Duration::from_millis(25));",  # watchdog: synthetic literal
    ]
    count, _, _mk = count_waits(lines, MODULE.RUST_PATTERNS)
    assert count == 1


def test_a_strict_elapsed_deadline_check_counts_but_a_floor_assertion_does_not() -> (
    None
):
    lines = [
        "    if started.elapsed() > deadline {",  # watchdog: synthetic literal
        "    assert!(started.elapsed() >= Duration::from_millis(50));",
    ]
    count, violations, _ = count_waits(lines, MODULE.RUST_PATTERNS)
    assert count == 1
    assert violations == ["1"]


def test_a_watchdog_marker_without_a_reason_does_not_pass() -> None:
    """The marker must carry a reason; a bare marker vouches for nothing."""
    for line in (
        "    time.sleep(0.05)  # watchdog:",  # watchdog: synthetic literal
        "    time.sleep(0.05)  # watchdog: x",  # watchdog: synthetic literal
    ):
        count, violations, _ = count_waits([line], [MODULE.PYTHON_PATTERNS[0]])
        assert count == 1, line
        assert violations == ["1"], line


def test_the_helpers_own_poll_sleep_is_marked() -> None:
    """The helper keeps its existing marked polling waits."""
    current = scan(ROOT)
    entry = current.get("tests/support.py")
    assert entry is not None and entry["count"] == 0, entry
    assert entry.get("markers") == 2, entry


def _planted_repo(tmp_path: Path, inventory: dict, files: dict[str, str]):
    """Plant a scratch repo. Keys are repo-relative paths like
    ``tests/test_one.py``."""
    (tmp_path / "tools").mkdir(parents=True)
    (tmp_path / "tools" / "test-wait-inventory.json").write_text(
        json.dumps(inventory), encoding="utf-8"
    )
    for name, body in files.items():
        target = tmp_path / name
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_text(body, encoding="utf-8")
    subprocess.run(["git", "init", "-q"], cwd=tmp_path, check=True)
    subprocess.run(["git", "add", "."], cwd=tmp_path, check=True)


def test_the_new_wait_forms_warn_on_planted_files(tmp_path: Path) -> None:
    """Each form the scanner recognizes warns in a planted repo."""
    root = tmp_path / "repo"
    _planted_repo(
        root,
        {"schema": "biomcp-test-wait-inventory-v1", "files": {}},
        {
            # Poll forms, not assert bounds: an assertion on a clock
            # value bounds duration and is exempt; a poll waits.
            "tests/test_elapsed_left.rs": (
                "#[test]\nfn t() {\n    if start.elapsed() < D(5) { break; }\n}\n"
            ),
            "tests/test_elapsed_right.rs": (
                "#[test]\nfn t() {\n    if deadline < start.elapsed() { break; }\n}\n"
            ),
            "tests/test_chained_elapsed.rs": (
                "#[test]\nfn t() {\n"
                "    while start.elapsed().as_millis() > 5 { break; }\n}\n"
            ),
            "tests/test_anyio.py": "def t():\n    await anyio.sleep(0.1)\n",  # watchdog: planted literal
            "tests/test_alias_time.py": (
                "import time as t\n\ndef test_waits():\n    t.sleep(0.1)\n"  # watchdog: planted literal
            ),
            "tests/test_alias_sleep.py": (
                "from time import sleep as snooze\n\ndef test_waits():\n"  # watchdog: planted literal
                "    snooze(0.1)\n"  # watchdog: planted literal
            ),
        },
    )
    result = _run(root)
    assert result.returncode == 0
    for name in (
        "tests/test_elapsed_left.rs",
        "tests/test_elapsed_right.rs",
        "tests/test_chained_elapsed.rs",
        "tests/test_anyio.py",
        "tests/test_alias_time.py",
        "tests/test_alias_sleep.py",
    ):
        assert name in result.stdout, (name, result.stdout)


def test_rust_sleep_until_and_elapsed_count() -> None:
    lines = [
        "    tokio::time::sleep_until(deadline).await;",
        "    if deadline < start.elapsed() { break; }",
        "    while start.elapsed().as_millis() > 5 { break; }",
        "    let x = start.elapsed();",  # no comparison: not a wait
        "    assert!(deadline < start.elapsed());",  # a bound, not a wait
    ]
    count, _, _ = MODULE.count_waits(lines, MODULE.RUST_PATTERNS)
    assert count == 3


def test_marker_growth_warns(tmp_path: Path) -> None:
    """Marker growth remains visible without failing acceptance."""
    root = tmp_path / "repo"
    _planted_repo(
        root,
        {
            "schema": "biomcp-test-wait-inventory-v1",
            "marker_total_ceiling": 1,
            "files": {
                "tests/test_one.py": {
                    "count": 0,
                    "language": "python",
                    "markers": 1,
                }
            },
        },
        {
            "tests/test_one.py": (
                "def t():\n"
                "    time.sleep(0.1)  # watchdog: first bounded poll interval\n"  # watchdog: planted literal
            ),
            "tests/test_two.py": (
                "def t():\n"
                "    time.sleep(0.2)  # watchdog: second marker raises the total\n"  # watchdog: planted literal
            ),
        },
    )
    result = _run(root)
    assert result.returncode == 0
    # Both the global growth and the unregistered marker file warn.
    assert "above the global ceiling 1" in result.stdout, result.stdout
    assert "not in the inventory" in result.stdout, result.stdout


def test_round_three_forms_count() -> None:
    """Ticket 1269: the neighboring spellings the prior ratchet missed."""
    rust = [
        "use std::thread::{sleep as nap};",
        "    nap(Duration::from_millis(5));",  # watchdog: planted literal
        "    let e = start.elapsed();",
        "    if e < limit { break; }",  # watchdog: planted literal
        "    if Instant::now().duration_since(start) < span { break; }",  # watchdog: planted literal
    ]
    text = "\n".join(rust)
    count, violations, _ = MODULE.count_waits(
        rust,
        MODULE.RUST_PATTERNS,
        MODULE.rust_sleep_aliases(text) + MODULE.rust_time_bindings(text),
    )
    # nap, the stored-binding compare, and duration_since all count.
    assert count == 3, (count, violations)

    python = [
        "import os, time as t",
        "    while time.monotonic() < deadline:",  # watchdog: planted literal
        "        t.sleep(0.05)",  # watchdog: planted literal
    ]
    count, violations, _ = MODULE.count_waits(
        python, MODULE.PYTHON_PATTERNS, MODULE.local_time_aliases("\n".join(python))
    )
    assert count == 2, (count, violations)


def test_historical_raises_and_review_wording_are_not_read(tmp_path: Path) -> None:
    root = tmp_path / "repo"
    _planted_repo(
        root,
        {
            "schema": "biomcp-test-wait-inventory-v1",
            "files": {"tests/test_one.py": {"count": 0}},
            "raises": [{"ticket": "9001", "from": "obsolete", "to": None}],
        },
        {"tests/test_one.py": "time.sleep(0.1)\n"},  # watchdog: planted literal
    )
    tickets = root / "sdlc/tickets"
    tickets.mkdir(parents=True)
    # Reading this directory as a ticket would fail. Historical records
    # cannot authenticate or block the warning.
    (tickets / "9001-old.md").mkdir()
    inventory = root / "tools/test-wait-inventory.json"
    original = inventory.read_bytes()
    result = _run(root)
    assert result.returncode == 0, result.stderr
    assert "warning:" in result.stdout
    assert "9001" not in result.stdout
    assert inventory.read_bytes() == original


def test_scanner_errors_fail(tmp_path: Path) -> None:
    root = tmp_path / "repo"
    _planted_repo(
        root,
        {"schema": "biomcp-test-wait-inventory-v1", "files": {}},
        {"tests/test_one.py": "pass\n"},
    )
    source = root / "tests/test_one.py"
    source.write_bytes(b"\xff")
    unreadable = _run(root)
    assert unreadable.returncode != 0
    assert "test-wait scan error:" in unreadable.stderr
    source.unlink()
    missing = _run(root)
    assert missing.returncode != 0
    assert "test-wait scan error:" in missing.stderr
    inventory = root / "tools/test-wait-inventory.json"
    inventory.write_text("{", encoding="utf-8")
    assert _run(root).returncode != 0
    inventory.write_text('["invalid"]', encoding="utf-8")
    assert _run(root).returncode != 0
    inventory.unlink()
    assert _run(root).returncode != 0


def test_enumeration_and_invocation_errors_fail(tmp_path: Path) -> None:
    root = tmp_path / "not-a-repo"
    (root / "tools").mkdir(parents=True)
    (root / "tools/test-wait-inventory.json").write_text(
        json.dumps({"schema": "biomcp-test-wait-inventory-v1", "files": {}})
    )
    result = _run(root)
    assert result.returncode != 0
    assert "test-wait scan error:" in result.stderr
    assert _run(root, "--update").returncode != 0
    assert _run(root, "--unknown-option").returncode != 0
