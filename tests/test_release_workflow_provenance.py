from __future__ import annotations

import subprocess
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[1]
WORKFLOWS = REPO_ROOT / ".github" / "workflows"
RELEASE_WORKFLOW = WORKFLOWS / "release.yml"
GUARD = REPO_ROOT / "scripts" / "release-disabled.sh"
DISABLED_MESSAGE = "release disabled until ticket 0957 installs the public-artifact gate"
PUBLISH_ROUTES = (
    "softprops/action-gh-release",
    "gh-action-pypi-publish",
    "docker/build-push-action",
    "mkdocs gh-deploy",
    "mcp-publisher publish",
    "git push",
)


def _workflow() -> str:
    return RELEASE_WORKFLOW.read_text(encoding="utf-8")


def test_release_guard_is_manual_read_only_and_fails_with_the_stable_message() -> None:
    workflow = _workflow()

    assert "workflow_dispatch:" in workflow
    assert "\n  release:" not in workflow
    assert "contents: read" in workflow
    assert "contents: write" not in workflow
    assert "packages: write" not in workflow
    assert "id-token: write" not in workflow
    assert "release-disabled:" in workflow
    assert "bash scripts/release-disabled.sh" in workflow

    result = subprocess.run(
        ["bash", str(GUARD)], text=True, capture_output=True, check=False
    )

    assert result.returncode != 0
    assert result.stdout == ""
    assert result.stderr.strip() == DISABLED_MESSAGE


def test_no_committed_workflow_or_guard_helper_can_publish_while_release_is_disabled() -> None:
    release_workflow = _workflow()

    assert release_workflow.count("- run:") == 1
    assert "bash scripts/release-disabled.sh" in release_workflow
    for route in PUBLISH_ROUTES:
        assert route not in release_workflow

    for workflow in WORKFLOWS.glob("*.yml"):
        text = workflow.read_text(encoding="utf-8")
        for route in PUBLISH_ROUTES:
            assert route not in text, f"{workflow.name} exposes publication through {route}"
