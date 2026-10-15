"""Golden output cases: each command's JSON and Markdown against committed files.

Every case runs the real binary against the replay server. The golden file
pins the command's standard output byte for byte; a deliberate format change
is `pytest tests/golden --update`, read the diff, commit. A case also asserts
that its requests replayed from disk: any miss names the recording to make.
"""

from __future__ import annotations

import difflib
import json
import os
import re
import subprocess
from dataclasses import dataclass
from pathlib import Path

import pytest

from replay_env import REPO_ROOT, ReplayEnvironment

pytestmark = [pytest.mark.needs_binary]  # docs-only CI excludes this module

CASES_DIR = Path(__file__).resolve().parent / "cases"
BINARY = Path(os.environ.get("BIOMCP_BIN", REPO_ROOT / "target/debug/biomcp"))
OVERRIDE_NAME_RE = re.compile(r'"(BIOMCP_[A-Z0-9_]+)"')
OVERRIDE_SUFFIXES = ("_BASE", "_BASE_URL", "_FIXTURE_ORIGIN", "_URL")
NON_PROVIDER_OVERRIDES = frozenset({"BIOMCP_HEALTH_PROBE_BASE"})
RECORD_PASSES = 3
RECORD_TIMEOUT_SECONDS = 600.0


@dataclass(frozen=True)
class Case:
    id: str
    argv: tuple[str, ...]


CASES = (
    Case("get-gene-braf", ("get", "gene", "BRAF")),
    Case("get-variant-braf-v600e", ("get", "variant", "BRAF V600E")),
    Case(
        "search-all-braf-melanoma",
        ("search", "all", "--gene", "BRAF", "--disease", "melanoma"),
    ),
    Case(
        "search-all-braf-melanoma-counts-only",
        ("search", "all", "--gene", "BRAF", "--disease", "melanoma", "--counts-only"),
    ),
)
FORMAT_SUFFIX = {"json": ".json", "markdown": ".md"}


def _golden_path(case: Case, output_format: str) -> Path:
    return CASES_DIR / f"{case.id}{FORMAT_SUFFIX[output_format]}"


def _failed_requests(run) -> list[dict]:
    return [
        entry
        for entry in run.request_log_lines
        if entry.get("result") in ("miss", "record-error")
    ]


def _run_case(
    replay: ReplayEnvironment, case: Case, output_format: str
) -> subprocess.CompletedProcess:
    """Run the case until its requests all replay, then return that run.

    In a recording run the first pass captures live replies; the last pass
    must be miss-free so a golden file never pins a live call. In a replay
    run one pass is enough and a miss fails the case.
    """
    argv = [*case.argv]
    if output_format == "json":
        argv.append("--json")
    timeout = RECORD_TIMEOUT_SECONDS if replay.record else 120.0
    run = replay.run(BINARY, tuple(argv), timeout=timeout)
    passes = 1
    while replay.record and _failed_requests(run) and passes < RECORD_PASSES:
        run = replay.run(BINARY, tuple(argv), timeout=timeout)
        passes += 1
    failures = _failed_requests(run)
    assert not failures, (
        f"{case.id} ({output_format}) made requests with no recorded reply:\n"
        + "\n".join(json.dumps(entry, sort_keys=True) for entry in failures)
    )
    assert run.process.returncode == 0, (
        f"{case.id} ({output_format}) exited {run.process.returncode}\n"
        f"stderr:\n{run.process.stderr}"
    )
    return run.process


def _diff(expected: str, actual: str, name: str) -> str:
    return "".join(
        difflib.unified_diff(
            expected.splitlines(keepends=True),
            actual.splitlines(keepends=True),
            fromfile=f"{name} (committed)",
            tofile=f"{name} (actual)",
        )
    )


@pytest.mark.parametrize(
    "case, output_format",
    [(case, fmt) for case in CASES for fmt in ("json", "markdown")],
    ids=[f"{case.id}-{fmt}" for case in CASES for fmt in ("json", "markdown")],
)
def test_golden_output(
    case: Case,
    output_format: str,
    replay: ReplayEnvironment,
    golden_update: bool,
) -> None:
    process = _run_case(replay, case, output_format)
    output = process.stdout
    if output_format == "json":
        json.loads(output)
    golden = _golden_path(case, output_format)
    if golden_update:
        golden.parent.mkdir(parents=True, exist_ok=True)
        golden.write_text(output, encoding="utf-8")
        return
    assert golden.is_file(), f"{golden} is missing; run: pytest tests/golden --update"
    expected = golden.read_text(encoding="utf-8")
    assert expected == output, _diff(expected, output, golden.name)


def test_replay_env_covers_every_provider_override(
    replay: ReplayEnvironment,
) -> None:
    """No provider override in src may bypass the replay server."""
    names = set()
    for path in sorted((REPO_ROOT / "src").rglob("*.rs")):
        for name in OVERRIDE_NAME_RE.findall(path.read_text(encoding="utf-8")):
            if name.endswith(OVERRIDE_SUFFIXES):
                names.add(name)
    uncovered = sorted(names - NON_PROVIDER_OVERRIDES - set(replay.provider_env))
    assert not uncovered, f"provider overrides missing from the replay env: {uncovered}"
