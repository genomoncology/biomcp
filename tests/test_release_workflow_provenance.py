from __future__ import annotations

import re
import subprocess
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[1]
WORKFLOWS = REPO_ROOT / ".github" / "workflows"
RELEASE_WORKFLOW = WORKFLOWS / "release.yml"
GUARD = REPO_ROOT / "scripts" / "release-disabled.sh"
DISABLED_MESSAGE = (
    "release disabled until ticket 0957 installs the public-artifact gate"
)
PUBLISH_ROUTES = (
    "softprops/action-gh-release",
    "gh-action-pypi-publish",
    "docker/build-push-action",
    "docker/login-action",
    "mcp-publisher publish",
    "actions/upload-artifact",
    "mkdocs gh-deploy",
    "maturin publish",
    "twine upload",
    "uv publish",
    "cargo publish",
    "gh release create",
    "git push",
)
WRITE_PERMISSIONS = ("write", "write-all")


def _workflow() -> str:
    return RELEASE_WORKFLOW.read_text(encoding="utf-8")


def _top_level_block(workflow: str, heading: str) -> str:
    match = re.search(
        rf"^{re.escape(heading)}:\n(.*?)(?=^[A-Za-z][A-Za-z0-9_-]*:\n|\Z)",
        workflow,
        flags=re.MULTILINE | re.DOTALL,
    )
    assert match is not None, f"missing top-level workflow key: {heading}"
    return match.group(1)


def test_release_guard_is_manual_read_only_and_fails_with_the_stable_message() -> None:
    workflow = _workflow()

    assert _top_level_block(workflow, "on").strip() == "workflow_dispatch:"
    assert _top_level_block(workflow, "permissions").strip() == "contents: read"

    jobs = _top_level_block(workflow, "jobs")
    assert re.findall(r"^  ([a-z][a-z0-9-]*):$", jobs, flags=re.MULTILINE) == [
        "release-disabled"
    ]
    assert re.findall(r"^      - uses: (.+)$", jobs, flags=re.MULTILINE) == [
        "actions/checkout@v4"
    ]
    assert re.findall(r"^      - run: (.+)$", jobs, flags=re.MULTILINE) == [
        "bash scripts/release-disabled.sh"
    ]

    result = subprocess.run(
        ["bash", str(GUARD)], text=True, capture_output=True, check=False
    )

    assert result.returncode != 0
    assert result.stdout == ""
    assert result.stderr.strip() == DISABLED_MESSAGE


def test_no_committed_workflow_or_guard_helper_can_publish_while_release_is_disabled() -> (
    None
):
    release_workflow = _workflow()

    for route in PUBLISH_ROUTES:
        assert route not in release_workflow

    for workflow in WORKFLOWS.glob("*.yml"):
        text = workflow.read_text(encoding="utf-8")
        for route in PUBLISH_ROUTES:
            assert route not in text, (
                f"{workflow.name} exposes publication through {route}"
            )
        for permission in WRITE_PERMISSIONS:
            assert not re.search(
                rf"(?m)^\s+[a-z-]+:\s*{re.escape(permission)}\s*$", text
            ), f"{workflow.name} grants {permission} permission"
