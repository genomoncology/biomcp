from __future__ import annotations

import re
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[1]
WORKFLOWS = REPO_ROOT / ".github" / "workflows"
RELEASE_WORKFLOW = WORKFLOWS / "release.yml"


def _job_block(workflow: str, job_name: str) -> str:
    match = re.search(
        rf"^  {re.escape(job_name)}:\n(.*?)(?=^  [A-Za-z0-9_-]+:\n|\Z)",
        workflow,
        flags=re.MULTILINE | re.DOTALL,
    )
    assert match is not None, f"missing workflow job {job_name}"
    return match.group(1)


def test_container_only_dispatch_gates_the_release_jobs() -> None:
    release = RELEASE_WORKFLOW.read_text(encoding="utf-8")
    container_only_gate = (
        "if: github.event_name == 'release' || inputs.container_only != true"
    )

    assert "container_only:" in release
    assert container_only_gate in _job_block(release, "build")
    assert container_only_gate in _job_block(release, "pypi-build")


def test_container_publish_platforms_and_latest_guard() -> None:
    container_publish = _job_block(
        RELEASE_WORKFLOW.read_text(encoding="utf-8"), "container-publish"
    )

    assert "platforms: linux/amd64,linux/arm64" in container_publish
    assert "gh release view" in container_publish
    assert '"$LATEST_RELEASE" != "$TAG"' in container_publish
    assert "docker buildx imagetools create --tag" in container_publish


def test_container_publish_checks_out_the_packaging_ref() -> None:
    container_publish = _job_block(
        RELEASE_WORKFLOW.read_text(encoding="utf-8"), "container-publish"
    )
    checkout_steps = [
        step
        for step in container_publish.split("\n      - ")
        if "uses: actions/checkout@v4" in step
    ]

    assert len(checkout_steps) == 1
    assert "ref:" not in checkout_steps[0]


def test_container_publish_resolves_the_revision_from_the_tag() -> None:
    container_publish = _job_block(
        RELEASE_WORKFLOW.read_text(encoding="utf-8"), "container-publish"
    )

    assert "SOURCE_SHA=" in container_publish
    assert "repos/${GITHUB_REPOSITORY}/commits/${TAG}" in container_publish
    assert "git rev-parse HEAD" not in container_publish


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
        if path == RELEASE_WORKFLOW:
            continue
        text = path.read_text(encoding="utf-8")
        assert not any(route in text for route in routes), path.name
