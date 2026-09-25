"""Contract tests for the test-wait ceiling ratchet."""

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
        [sys.executable, str(ROOT / "tools" / "check-test-wait-ratchet.py"), "--root", str(root), *args],
        capture_output=True,
        text=True,
        check=False,
        cwd=ROOT,
    )


def test_inventory_counts_match_the_tree() -> None:
    """The real tree passes against its pinned ceilings."""
    result = _run(ROOT)
    assert result.returncode == 0, result.stdout + result.stderr


def test_a_new_unmarked_wait_fails(tmp_path: Path) -> None:
    """A synthetic unmarked time.sleep in a scanned test file fails."""  # watchdog: synthetic literal
    root = tmp_path / "repo"
    (root / "tools").mkdir(parents=True)
    (root / "tools" / "test-wait-inventory.json").write_text(
        json.dumps({"schema": "biomcp-test-wait-inventory-v1", "files": {}}),
        encoding="utf-8",
    )
    tests_dir = root / "tests"
    tests_dir.mkdir()
    (tests_dir / "test_example.py").write_text(
        "def test_waits() -> None:\n    time.sleep(0.1)\n", encoding="utf-8"  # watchdog: synthetic literal
    )
    # scan() shells out to git ls-files, so stage the file.
    subprocess.run(["git", "init", "-q"], cwd=root, check=True)
    subprocess.run(["git", "add", "."], cwd=root, check=True)

    result = _run(root)
    assert result.returncode == 1
    assert "new unmarked timed waits in tests/test_example.py (1)" in result.stdout


def test_a_watchdog_marker_passes() -> None:
    lines = ["    time.sleep(0.05)  # watchdog: shared poll interval inside the helper"]
    count, violations = count_waits(lines, [MODULE.PYTHON_PATTERNS[0]])
    assert count == 0
    assert violations == []


def test_a_heartbeat_helper_with_a_bare_sleep_counts() -> None:
    lines = [
        "def _heartbeat_advances(path):",
        "    before = path.read_text()",
        "    time.sleep(0.2)",  # watchdog: synthetic literal
        "    return path.read_text() != before",
    ]
    count, violations = count_waits(lines, [MODULE.PYTHON_PATTERNS[0]])
    assert count == 1
    assert violations == ["3 heartbeat helper contains a bare timed wait"]


def test_watchdog_builder_deadlines_do_not_count() -> None:
    lines = [
        "    let deadline = std::time::Instant::now() + crate::test_support::watchdog(30);",
        "    let deadline = tokio::time::Instant::now() + watchdog(60);",
        "    let deadline = tokio::time::Instant::now() + Duration::from_secs(60);",
    ]
    count, violations = count_waits(lines, MODULE.RUST_PATTERNS)
    assert count == 1
    assert violations == ["3"]


def test_an_inventory_decrease_passes_and_notes_the_ratchet_down(tmp_path: Path) -> None:
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
        "def test_waits() -> None:\n    time.sleep(0.1)\n", encoding="utf-8"  # watchdog: synthetic literal
    )
    subprocess.run(["git", "init", "-q"], cwd=root, check=True)
    subprocess.run(["git", "add", "."], cwd=root, check=True)

    result = _run(root)
    assert result.returncode == 0, result.stdout + result.stderr
    assert "dropped 2 -> 1" in result.stdout
    assert "re-pin down with --update" in result.stdout


def test_the_helpers_own_poll_sleep_is_marked() -> None:
    """tests/support.py passes the ratchet because its sleep carries the marker."""
    current = scan(ROOT)
    assert "tests/support.py" not in current
