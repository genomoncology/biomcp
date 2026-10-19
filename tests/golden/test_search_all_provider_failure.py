"""Synthetic Reactome 500 scenario preserves search-all failure behavior.

The normal replay uses the observed successful BRAF reply. This scenario
explicitly injects HTTP 500 with the historical error body. Its manifest
exists only in the isolated test root and never enters live drift checks.
"""

from __future__ import annotations

import json
from pathlib import Path

import pytest

from replay_env import REPO_ROOT, replay_environment
from test_golden_output import (
    CASES,
    CASES_DIR,
    FORMAT_SUFFIX,
    Case,
    _diff,
    _normalize,
    _run_case,
)

pytestmark = pytest.mark.needs_binary
FAILURE_BODY = REPO_ROOT / "tests/fixtures/pathway/synthetic-reactome-failure/body.json"
BRAF_KEY = "GET /reactome/search/query?pageSize=3&query=BRAF&species=Homo+sapiens"
SEARCH_ALL = [case for case in CASES if case.id.startswith("search-all-")]


@pytest.mark.parametrize("case", SEARCH_ALL, ids=lambda case: case.id)
@pytest.mark.parametrize("output_format", ("json", "markdown"))
def test_search_all_preserves_synthetic_reactome_failure(
    tmp_path: Path,
    case: Case,
    output_format: str,
) -> None:
    root = tmp_path / "synthetic-reactome-failure"
    root.mkdir()
    (root / "src").symlink_to(REPO_ROOT / "src", target_is_directory=True)
    replay = REPO_ROOT / "testdata/replay"
    projected = root / "testdata/replay"
    projected.mkdir(parents=True)
    manifest = json.loads((replay / "manifest.json").read_text())
    for key, route in manifest["routes"].items():
        target = projected / route["file"]
        target.parent.mkdir(parents=True, exist_ok=True)
        if not target.exists():
            target.symlink_to(
                FAILURE_BODY if key == BRAF_KEY else replay / route["file"]
            )
    manifest["routes"][BRAF_KEY]["status"] = 500
    (projected / "manifest.json").write_text(json.dumps(manifest))
    scenario = Case(f"{case.id}-synthetic-reactome-failure", case.argv)
    with replay_environment(root=root) as environment:
        process = _run_case(environment, scenario, output_format)
    expected_path = CASES_DIR / f"{scenario.id}{FORMAT_SUFFIX[output_format]}"
    expected = expected_path.read_text()
    output = _normalize(process.stdout)
    assert expected == output, _diff(expected, output, expected_path.name)
