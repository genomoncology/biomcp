"""Exercise measurement summaries through the coverage command entry point."""

from __future__ import annotations

import os
import shutil
import subprocess
from pathlib import Path

import pytest

ROOT = Path(__file__).resolve().parents[1]


def executable(path: Path, body: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text("#!/usr/bin/env bash\nset -eu\n" + body)
    path.chmod(0o755)


@pytest.mark.parametrize(
    ("nextest_summary", "spec_summary", "status", "expected", "incomplete"),
    [
        (
            "Summary [ 12.0s] 4183 tests run: 4183 passed, 2 skipped",
            "=== spec pages: spec/a.md spec/b.md spec/c.md ===",
            "0",
            ("4183 run: 4183 passed", "3 pages, 0 failed"),
            False,
        ),
        (
            "Summary [  24.364s] 1841/4183 tests run: 1840 passed, 1 failed, 29 skipped",
            "=== spec pages: spec/a.md spec/b.md spec/c.md ===",
            "100",
            ("1841 run: 1840 passed, 1 failed", "3 pages, 0 failed"),
            True,
        ),
        (
            "Summary [ 1.0s] 3 tests run: 3 passed",
            "=== spec page: spec/a.md ===\n=== spec page: spec/b.md ===",
            "0",
            ("3 run: 3 passed", "2 pages, 0 failed"),
            False,
        ),
        (
            "Summary [ 1.0s] 3 tests run: 3 passed",
            "=== spec pages: spec/a.md spec/b.md ===\nspec run failed (1)",
            "0",
            ("3 run: 3 passed", "2 pages, 1 failed"),
            True,
        ),
        (
            "",
            "=== spec pages: spec/a.md ===",
            "0",
            ("summary unparsed", "1 pages, 0 failed"),
            True,
        ),
        (
            "Summary [ 1.0s] 3 tests run: 3 passed",
            "",
            "0",
            ("3 run: 3 passed", "summary unparsed"),
            True,
        ),
    ],
)
def test_coverage_summary_counts_executed_work(
    tmp_path: Path,
    nextest_summary: str,
    spec_summary: str,
    status: str,
    expected: tuple[str, str],
    incomplete: bool,
) -> None:
    bash = shutil.which("bash") or "bash"
    version = subprocess.run(
        [bash, "-c", '[[ "${BASH_VERSINFO[0]}" -ge 4 ]]'],
        check=False,
    )
    if version.returncode:
        pytest.skip("coverage measurement requires Bash 4 or newer on PATH")
    clone = tmp_path / "clone"
    clone.mkdir()
    fake_home = tmp_path / "home"
    executable(
        fake_home / ".cargo/bin/cargo",
        """case "$*" in
  "llvm-cov report"*) echo 'TOTAL 0 0 0 0 0 0 0 0 72.5%' ;;
  "nextest run"*) printf '%s\\n' "$TEST_NEXTEST_SUMMARY"; exit "$TEST_SUITE_STATUS" ;;
esac
""",
    )
    executable(
        fake_home / ".local/bin/uv",
        "echo '================ 5 passed in 0.1s ================'\n",
    )
    executable(clone / "tools/run-offline", 'shift; exec "$@"\n')
    executable(
        clone / "scripts/run-specs.sh",
        'printf "%s\\n" "$TEST_SPEC_SUMMARY"\n',
    )
    env = dict(os.environ)
    env.update(
        HOME=str(fake_home),
        TEST_NEXTEST_SUMMARY=nextest_summary,
        TEST_SPEC_SUMMARY=spec_summary,
        TEST_SUITE_STATUS=status,
    )
    proc = subprocess.run(
        [
            bash,
            str(ROOT / "tools/measure-0-9-3-baseline"),
            str(clone),
            "--only",
            "coverage",
        ],
        env=env,
        text=True,
        capture_output=True,
        timeout=20,
    )
    assert proc.returncode == 0, proc.stdout + proc.stderr
    lines = dict(
        line.removeprefix("RESULT ").split(" = ", 1)
        for line in proc.stdout.splitlines()
        if line.startswith("RESULT ")
    )
    assert expected[0] in lines["coverage_unit_tests"], proc.stdout
    assert expected[1] in lines["coverage_spec_lane"], proc.stdout
    if incomplete:
        assert lines["coverage_combined_lines_pct"].startswith("incomplete ("), (
            proc.stdout
        )
    else:
        assert lines["coverage_combined_lines_pct"] == "72.5", proc.stdout
