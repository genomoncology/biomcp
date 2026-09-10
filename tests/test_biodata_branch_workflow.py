from __future__ import annotations

from pathlib import Path
import re
import os
import subprocess
import tempfile

import pytest


ROOT = Path(__file__).resolve().parents[1]
RETIRED_WORKFLOW = ROOT / ".github/workflows/biodata-1.0.yml"
RUNNER = ROOT / "tools/check-biodata-1.0"
TEST_FILES = (
    "tests/test_biodata_boundary.py",
    "tests/test_source_package_boundary.py",
    "tests/test_ctgov_trial_search_detail_reuse.py",
    "tests/test_nci_filter_transport.py",
    "tests/test_biodata_branch_workflow.py",
)
EXPECTED_ISOLATED_INVOCATION = (
    '"$ROOT/tools/check-offline-network" true\n'
    'python3 "$ROOT/tools/check-biodata-boundary.py" --root "$ROOT"\n'
    'exec env -u NCI_API_KEY BIOMCP_BIN="$biomcp_bin" \\\n'
    '  uv run --no-sync pytest --basetemp /tmp/pytest "${TEST_FILES[@]}"\n'
)
FORBIDDEN_COMMANDS = (
    "make lint",
    "make test",
    "make spec",
    "make full-feature-check",
    "make release-gate",
    "curl ",
    "wget ",
    "uv publish",
    "cargo publish",
    "git push",
    "deploy",
)


def _runner_test_files(runner: str) -> tuple[str, ...]:
    match = re.search(r"readonly TEST_FILES=\(\n(?P<body>.*?)\n\)", runner, re.DOTALL)
    if match is None:
        return ()
    return tuple(re.findall(r'^\s+"([^"]+)"$', match.group("body"), re.MULTILINE))


def _runner_violations(runner: str) -> list[str]:
    violations: list[str] = []
    for value in (
        "unset NCI_API_KEY",
        "env -u NCI_API_KEY",
        "tools/check-biodata-boundary.py",
        "tools/check-offline-network",
        '[[ ! -x "$biomcp_bin" ]]',
    ):
        if value not in runner:
            violations.append(f"missing focused runner control: {value}")
    if runner.count(EXPECTED_ISOLATED_INVOCATION) != 1:
        violations.append("isolated focused runner invocation changed")
    if "tools/run-offline" in runner or "bwrap" in runner:
        violations.append("BioMCP runner must not establish isolation")
    if '[[ "${BIOMCP_OFFLINE_NETWORK:-0}" != 1 ]]' not in runner:
        violations.append("already-isolated mode must require the verified namespace")
    if _runner_test_files(runner) != TEST_FILES:
        violations.append("focused runner test selection changed")
    if "cargo build" in runner or "cargo run" in runner:
        violations.append("focused runner must require an existing binary")
    for command in FORBIDDEN_COMMANDS:
        if command in runner.lower():
            violations.append(f"forbidden focused runner command: {command}")
    return violations


def test_biomcp_keeps_only_the_local_focused_runner() -> None:
    assert not RETIRED_WORKFLOW.exists()
    assert not any(
        "biodata/biomcp-1.0" in path.read_text(encoding="utf-8")
        for path in (ROOT / ".github/workflows").iterdir()
        if path.suffix in {".yml", ".yaml"}
    )
    runner = RUNNER.read_text(encoding="utf-8")
    assert not _runner_violations(runner)
    assert RUNNER.stat().st_mode & 0o111


@pytest.mark.parametrize(
    ("old", "new"),
    (
        ("unset NCI_API_KEY", "true # retain NCI_API_KEY"),
        ("tools/check-offline-network", "true # trust marker"),
        (TEST_FILES[2], "tests/test_live_provider.py"),
        ("uv run --no-sync pytest", "make test && uv run --no-sync pytest"),
        ('[[ ! -x "$biomcp_bin" ]]', '[[ -x "$biomcp_bin" ]]'),
        ("--basetemp /tmp/pytest", "--basetemp /var/tmp/pytest"),
        (' "${TEST_FILES[@]}"', ""),
        (' "${TEST_FILES[@]}"', ' "${TEST_FILES[@]}" tests/'),
    ),
)
def test_representative_runner_mutations_are_rejected(old: str, new: str) -> None:
    runner = RUNNER.read_text(encoding="utf-8")
    assert old in runner
    assert _runner_violations(runner.replace(old, new, 1))


def test_forged_offline_marker_fails_in_the_normal_namespace() -> None:
    cache = ROOT / ".cache"
    cache.mkdir(exist_ok=True)
    with tempfile.TemporaryDirectory(dir=cache) as directory:
        binary = Path(directory) / "biomcp"
        binary.write_text("#!/usr/bin/env bash\nexit 0\n", encoding="utf-8")
        binary.chmod(0o755)
        environment = os.environ.copy()
        environment.update(
            BIOMCP_BIN=str(binary),
            BIOMCP_OFFLINE_NETWORK="1",
        )
        result = subprocess.run(
            [str(RUNNER), "--already-isolated"],
            cwd=ROOT,
            env=environment,
            capture_output=True,
            text=True,
        )
    assert result.returncode != 0
    assert "offline privilege isolation failed" in result.stdout + result.stderr
