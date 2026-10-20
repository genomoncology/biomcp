"""Golden-suite fixtures: the --update flag and the shared replay server."""

from __future__ import annotations

import os
import json
from typing import Iterator

import sys
from pathlib import Path

import pytest

_HERE = Path(__file__).resolve().parent
if str(_HERE) not in sys.path:
    sys.path.insert(0, str(_HERE))


@pytest.fixture
def replay(request: pytest.FixtureRequest, tmp_path: Path) -> Iterator["object"]:
    """One isolated view of the session's replay server for each golden case.

    Set BIOMCP_GOLDEN_RECORD=1 for a run that records live replies for
    unrecorded requests; without it every request must replay from disk.
    """
    from replay_env import REPO_ROOT, replay_environment

    params = getattr(getattr(request.node, "callspec", None), "params", {})
    case = params.get("case")
    oa_case = case is not None and case.id in {
        "get-article-23056577-fulltext", "get-article-23056577-assets",
    }
    oa_bytes = request.node.name == "test_article_supplement_stdout_preserves_recorded_bytes"
    root = None
    if oa_case or oa_bytes:
        # This authored availability control is deliberately outside the live
        # capture inventory. Both real ZIP attempts timed out without a body;
        # HTTP504 approximates that failure, not an observed provider status.
        root = tmp_path / "oa-secondary-unavailable"
        replay_dir = root / "testdata/replay"
        replay_dir.mkdir(parents=True)
        (root / "src").symlink_to(REPO_ROOT / "src", target_is_directory=True)
        (replay_dir / "responses").symlink_to(
            REPO_ROOT / "testdata/replay/responses", target_is_directory=True,
        )
        body = replay_dir / "authored-secondary-unavailable.json"
        body.write_text('{"authored":true,"error":"Secondary ZIP unavailable after timeout"}\n')
        manifest = json.loads((REPO_ROOT / "testdata/replay/manifest.json").read_text())
        manifest["routes"]["GET /europepmc/PMC3467229/supplementaryFiles"] = {
            "status": 504,
            "file": body.name,
            "headers": {"content-type": "application/json"},
            "note": "Authored timeout approximation; no observed complete ZIP response.",
        }
        (replay_dir / "manifest.json").write_text(json.dumps(manifest))

    record = os.environ.get("BIOMCP_GOLDEN_RECORD") == "1"
    with replay_environment(record=record, root=root) as environment:
        yield environment


@pytest.fixture(scope="session")
def golden_update(request: pytest.FixtureRequest) -> bool:
    return bool(request.config.getoption("--update"))
