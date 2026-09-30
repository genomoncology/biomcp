"""Behavioral tests for scripts/should-move-latest.sh.

The 2026-09-30 second go-request review found the release blocker:
the `latest` move asked GitHub for "the latest release", which skips
drafts, so a draft v0.9.1 read v0.9.0 as latest and the step would
silently skip. The decision is now a pure version comparison against
the PUBLISHED tags.
"""

from __future__ import annotations

import os
import shutil
import subprocess
from pathlib import Path

import pytest
import yaml

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


def test_a_two_digit_minor_sorts_newer_than_v9() -> None:
    """Plain text comparison would call v0.10.0 older than v0.9.0
    ("1" < "9"); the numeric key must not (third go-request review).
    """
    ok, _ = _decide("v0.10.0", ["v0.9.0", "v0.9.9"])
    assert ok
    ok_back, _ = _decide("v0.9.9", ["v0.10.0"])
    assert not ok_back


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


@pytest.mark.parametrize('arguments', [
    ['v0.8.9-rc1'], ['v0.8.9', 'not-a-tag'],
])
def test_invalid_helper_input_has_an_error_status(arguments: list[str]) -> None:
    result = subprocess.run(
        ['bash', str(SCRIPT), *arguments], capture_output=True, text=True, check=False,
    )
    assert result.returncode == 2


def _publish_steps() -> list[dict]:
    workflow = yaml.safe_load(
        (REPO_ROOT / '.github/workflows/release.yml').read_text(encoding='utf-8')
    )
    return workflow['jobs']['publish-release']['steps']


def test_publish_checks_out_the_releasing_sha_before_running_the_helper() -> None:
    steps = _publish_steps()
    checkout = steps[0]
    assert checkout['uses'] == 'actions/checkout@11bd71901bbe5b1630ceea73d27597364c9af683'
    assert checkout['with']['ref'] == '${{ github.sha }}'


@pytest.mark.parametrize(
    ('listing_status', 'tags', 'helper', 'expected_status', 'moves', 'publishes'),
    [
        (0, '', 'real', 0, True, True),
        (0, 'v0.8.9\nv0.8.1\n', 'real', 0, True, True),
        (0, 'v0.9.0\n', 'real', 0, False, True),
        (0, 'v0.9.0-rc1\n', 'real', 0, True, True),
        (1, '', 'real', 1, False, False),
        (1, 'v0.8.1\n', 'real', 1, False, False),
        (0, '', 'missing', 1, False, False),
        (0, '', 'error', 42, False, False),
    ],
)
def test_publish_workflow_stops_on_query_or_helper_failure(
    tmp_path: Path, listing_status: int, tags: str, helper: str,
    expected_status: int, moves: bool, publishes: bool,
) -> None:
    """Execute the actual workflow run steps with only gh/docker replaced."""
    commands = tmp_path / 'bin'
    commands.mkdir()
    log = tmp_path / 'calls'
    gh = commands / 'gh'
    gh.write_text(
        '#!/bin/bash\nprintf "gh %s\\n" "$*" >> "$CALL_LOG"\n'
        'if [ "$1 $2" = "release list" ]; then\n'
        '  printf "%s" "$LIST_TAGS"\n  exit "$LIST_STATUS"\nfi\n',
        encoding='utf-8',
    )
    docker = commands / 'docker'
    docker.write_text(
        '#!/bin/bash\nprintf "docker %s\\n" "$*" >> "$CALL_LOG"\n',
        encoding='utf-8',
    )
    gh.chmod(0o755)
    docker.chmod(0o755)
    scripts = tmp_path / 'scripts'
    scripts.mkdir()
    if helper != 'missing':
        shutil.copyfile(SCRIPT, scripts / 'decision.sh')
        target = scripts / 'should-move-latest.sh'
        target.write_text(
            '#!/bin/bash\necho helper >> "$CALL_LOG"\n' + (
                'exit 42\n' if helper == 'error' else
                'exec bash scripts/decision.sh "$@"\n'
            ), encoding='utf-8',
        )
        target.chmod(0o755)
    env = {
        **os.environ, 'PATH': f'{commands}:/usr/bin:/bin',
        'CALL_LOG': str(log), 'LIST_TAGS': tags,
        'LIST_STATUS': str(listing_status), 'TAG': 'v0.8.9',
        'GITHUB_REPOSITORY': 'example/consumer',
        'RUNNER_TEMP': str(tmp_path),
    }
    status = 0
    for step in _publish_steps():
        if 'run' not in step:
            continue
        result = subprocess.run(
            ['bash', '--noprofile', '--norc', '-e', '-o', 'pipefail', '-c', step['run']],
            cwd=tmp_path, env=env, capture_output=True, text=True, check=False,
        )
        status = result.returncode
        if status:
            break
    calls = log.read_text() if log.exists() else ''
    assert status == expected_status, result.stderr
    assert ('docker buildx imagetools create' in calls) == moves
    assert ('docker buildx imagetools inspect' in calls) == moves
    assert ('gh release edit v0.8.9 --draft=false' in calls) == publishes
    if helper == 'real' and listing_status == 0:
        assert 'helper\n' in calls
