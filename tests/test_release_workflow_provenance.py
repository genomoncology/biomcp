from __future__ import annotations

import re
from pathlib import Path

import pytest

REPO_ROOT = Path(__file__).resolve().parents[1]
WORKFLOWS = REPO_ROOT / ".github" / "workflows"
RELEASE_WORKFLOW = WORKFLOWS / "release.yml"

EXPECTED_NEEDS = {
    "version-check": [],
    "create-draft": ["version-check"],
    "build": ["version-check", "create-draft"],
    "pypi-build": ["version-check"],
    "wheel-smoke": ["pypi-build"],
    "docs-live": ["version-check"],
    "pypi-publish": ["pypi-build", "wheel-smoke", "docs-live"],
    "homebrew-tap": ["build", "docs-live", "wheel-smoke"],
    "container-publish": [
        "build",
        "docs-live",
        "version-check",
        "wheel-smoke",
        "create-draft",
    ],
    "publish-release": [
        "build",
        "pypi-publish",
        "homebrew-tap",
        "container-publish",
        "docs-live",
    ],
}


def _job_block(workflow: str, job: str) -> str:
    match = re.search(
        rf"^  {re.escape(job)}:\n(.*?)(?=^  [A-Za-z0-9_-]+:\n|\Z)",
        workflow,
        re.MULTILINE | re.DOTALL,
    )
    assert match is not None, f"missing workflow job {job}"
    return match.group(1)


def _needs(block: str) -> list[str]:
    match = re.search(r"^    needs: \[([^]]*)\]$", block, re.MULTILINE)
    if match is None:
        return []
    return [part.strip() for part in match.group(1).split(",") if part.strip()]


def _assert_release_contract(workflow: str) -> None:
    assert "push:\n    tags: ['v*']" in workflow
    assert "permissions: {}" in workflow
    assert "group: release-${{ inputs.tag || github.ref_name }}" in workflow
    assert "github.event.release.tag_name" not in workflow
    for job, needs in EXPECTED_NEEDS.items():
        assert _needs(_job_block(workflow, job)) == needs, job

    expected_ifs = {
        "create-draft": "if: github.event_name == 'push'",
        "build": "if: github.event_name == 'push'",
        "pypi-build": "if: github.event_name == 'push'",
        "wheel-smoke": "if: github.event_name == 'push'",
        "pypi-publish": "if: github.event_name == 'push'",
        "homebrew-tap": "if: github.event_name == 'push'",
        "publish-release": "if: github.event_name == 'push'",
    }
    for job, condition in expected_ifs.items():
        assert condition in _job_block(workflow, job), job
    assert "\n    if:" not in _job_block(workflow, "version-check")
    assert "\n    if:" not in _job_block(workflow, "docs-live")

    container = _job_block(workflow, "container-publish")
    for clause in (
        "!cancelled()",
        "github.event_name == 'push' && success()",
        "inputs.container_only == true",
        "needs.version-check.result == 'success'",
        "needs.docs-live.result == 'success'",
        "needs.create-draft.result == 'skipped'",
        "needs.build.result == 'skipped'",
        "needs['wheel-smoke'].result == 'skipped'",
    ):
        assert clause in container
    assert "always()" not in workflow

    version = _job_block(workflow, "version-check")
    assert "fetch-depth: 0" in version
    assert "check-release-versions.py" in version
    assert "check-changelog-coverage.py" in version
    assert "continue-on-error" not in version
    assert "|| true" not in version
    assert "if: false" not in version

    permissions = {
        "version-check": "contents: read",
        "create-draft": "contents: write",
        "build": "contents: write",
        "pypi-build": "contents: read",
        "docs-live": "contents: read",
        "homebrew-tap": "contents: read",
        "pypi-publish": "id-token: write",
        "container-publish": "packages: write",
        "publish-release": "contents: write",
    }
    for job, grant in permissions.items():
        assert grant in _job_block(workflow, job), job
    assert "permissions:" not in _job_block(workflow, "wheel-smoke")

    draft = _job_block(workflow, "create-draft")
    assert 'gh release create "$TAG" --draft --verify-tag' in draft
    build = _job_block(workflow, "build")
    assert 'gh release upload "$TAG"' in build and "--clobber" in build
    assert "skip-existing" not in workflow and "skip_existing" not in workflow

    smoke = _job_block(workflow, "wheel-smoke")
    for artifact in (
        "wheel-x86_64-unknown-linux-gnu",
        "wheel-aarch64-unknown-linux-gnu",
        "wheel-aarch64-apple-darwin",
        "wheel-x86_64-apple-darwin",
        "wheel-x86_64-pc-windows-msvc",
    ):
        assert artifact in smoke
    assert "Scripts/biomcp.exe" in smoke and "bin/biomcp" in smoke
    assert '"$status" -eq 101' in smoke and '"$status" -ge 128' in smoke
    assert "require_exit 0 drug interactions apixaban" in smoke
    assert "Drug not found in FAERS" in smoke
    assert "skill asset" in smoke

    final = _job_block(workflow, "publish-release")
    assert 'gh release edit "$TAG" --draft=false' in final
    assert "gh release view" in final
    assert "docker buildx imagetools create --tag" in final
    assert "gh release view" not in container
    assert "imagetools create --tag" not in container


def test_release_workflow_contract() -> None:
    _assert_release_contract(RELEASE_WORKFLOW.read_text(encoding="utf-8"))


@pytest.mark.parametrize(
    "job,edge", [(job, edge) for job, edges in EXPECTED_NEEDS.items() for edge in edges]
)
def test_removing_each_needs_edge_breaks_the_contract(
    tmp_path: Path, job: str, edge: str
) -> None:
    workflow = RELEASE_WORKFLOW.read_text(encoding="utf-8")
    block = _job_block(workflow, job)
    mutated = (
        block.replace(edge + ", ", "", 1)
        .replace(", " + edge, "", 1)
        .replace("[" + edge + "]", "[]", 1)
    )
    assert mutated != block
    scratch = tmp_path / "release.yml"
    scratch.write_text(workflow.replace(block, mutated, 1), encoding="utf-8")
    with pytest.raises(AssertionError):
        _assert_release_contract(scratch.read_text(encoding="utf-8"))


@pytest.mark.parametrize(
    "needle,replacement",
    [
        (
            'run: python3 scripts/check-release-versions.py --tag "$TAG"',
            'run: python3 scripts/check-release-versions.py --tag "$TAG" || true',
        ),
        ("    steps:\n", "    continue-on-error: true\n    steps:\n"),
        (
            "      - name: Require the tag and committed versions to agree\n",
            "      - name: Require the tag and committed versions to agree\n        if: false\n",
        ),
    ],
)
def test_neutering_a_gate_breaks_the_contract(
    tmp_path: Path, needle: str, replacement: str
) -> None:
    workflow = RELEASE_WORKFLOW.read_text(encoding="utf-8")
    assert needle in workflow
    scratch = tmp_path / "release.yml"
    scratch.write_text(workflow.replace(needle, replacement, 1), encoding="utf-8")
    with pytest.raises(AssertionError):
        _assert_release_contract(scratch.read_text(encoding="utf-8"))


@pytest.mark.parametrize(
    "clause",
    [
        "success()",
        "inputs.container_only == true",
        "needs.version-check.result == 'success'",
        "needs.docs-live.result == 'success'",
        "needs.create-draft.result == 'skipped'",
        "needs.build.result == 'skipped'",
        "needs['wheel-smoke'].result == 'skipped'",
    ],
)
def test_removing_a_container_condition_clause_breaks_the_contract(
    tmp_path: Path, clause: str
) -> None:
    workflow = RELEASE_WORKFLOW.read_text(encoding="utf-8")
    scratch = tmp_path / "release.yml"
    scratch.write_text(workflow.replace(clause, "true", 1), encoding="utf-8")
    with pytest.raises(AssertionError):
        _assert_release_contract(scratch.read_text(encoding="utf-8"))


def test_actions_are_pinned_and_pypi_uses_trusted_publishing() -> None:
    workflow = RELEASE_WORKFLOW.read_text(encoding="utf-8")
    assert not re.findall(r"uses: [^\s]+@(?![0-9a-f]{40}\b)[^\s]+", workflow)
    pypi = _job_block(workflow, "pypi-publish")
    assert "environment: pypi" in pypi and "id-token: write" in pypi
    assert "uv publish --trusted-publishing always dist/*" in pypi


def test_no_other_workflow_exposes_release_publication() -> None:
    routes = (
        "gh release create",
        "uv publish",
        "skopeo copy",
        "git push",
        "docker push",
        "imagetools create",
    )
    for path in WORKFLOWS.glob("*.yml"):
        if path != RELEASE_WORKFLOW:
            text = path.read_text(encoding="utf-8")
            assert not any(route in text for route in routes), path.name
