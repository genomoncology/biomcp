from __future__ import annotations

from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[1]
WORKFLOWS = REPO_ROOT / ".github" / "workflows"
RELEASE_WORKFLOW = WORKFLOWS / "release.yml"


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
