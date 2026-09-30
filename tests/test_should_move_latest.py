"""Behavioral tests for scripts/should-move-latest.sh.

The 2026-09-30 second go-request review found the release blocker:
the `latest` move asked GitHub for "the latest release", which skips
drafts, so a draft v0.9.1 read v0.9.0 as latest and the step would
silently skip. The decision is now a pure version comparison against
the PUBLISHED tags.
"""

from __future__ import annotations

import subprocess
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[1]
SCRIPT = REPO_ROOT / "scripts" / "should-move-latest.sh"


def _decide(publishing: str, published: list[str]) -> tuple[bool, str]:
    result = subprocess.run(
        ["bash", str(SCRIPT), publishing, *published],
        capture_output=True,
        text=True,
        check=False,
    )
    return result.returncode == 0, (result.stderr or result.stdout).strip()


def test_a_draft_newer_than_every_published_release_moves_latest() -> None:
    ok, _ = _decide("v0.9.1", ["v0.9.0", "v0.8.25", "v0.8.9"])
    assert ok


def test_an_older_tag_never_moves_latest() -> None:
    ok, reason = _decide("v0.8.9", ["v0.9.0"])
    assert not ok
    assert "sorts newer" in reason


def test_no_published_releases_moves_latest() -> None:
    ok, _ = _decide("v0.1.0", [])
    assert ok


def test_an_unstable_tag_is_refused() -> None:
    ok, reason = _decide("v0.9.1-rc1", ["v0.9.0"])
    assert not ok
    assert "not a stable tag" in reason


def test_equality_still_moves_latest() -> None:
    ok, _ = _decide("v0.9.1", ["v0.9.1"])
    assert ok
