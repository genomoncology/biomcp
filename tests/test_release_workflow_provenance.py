from __future__ import annotations

import re
from pathlib import Path

import yaml


REPO_ROOT = Path(__file__).resolve().parents[1]
WORKFLOWS = REPO_ROOT / ".github" / "workflows"
RELEASE_WORKFLOW = WORKFLOWS / "release.yml"


def _workflow() -> dict:
    return yaml.safe_load(RELEASE_WORKFLOW.read_text(encoding="utf-8"))


def test_write_permissions_exist_only_on_the_two_protected_pointer_paths() -> None:
    workflow = _workflow()
    assert workflow["permissions"] == {"contents": "read"}
    writers = {
        name
        for name, job in workflow["jobs"].items()
        if "write" in str(job.get("permissions", {}))
    }
    assert writers == {"publish-versioned", "advance-mutable-pointers"}
    for name in writers:
        assert workflow["jobs"][name]["environment"] == "biomcp-release-promotion"


def test_stage_is_read_only_and_latest_waits_for_public_reconciliation() -> None:
    workflow = _workflow()
    stage_jobs = {
        name: job
        for name, job in workflow["jobs"].items()
        if job.get("if") == "inputs.mode == 'stage'"
    }
    assert set(stage_jobs) == {
        "candidate-gates",
        "linux-artifacts",
        "signed-artifacts",
        "container-artifact",
        "homebrew-formula",
        "homebrew-smoke",
        "mcpb-artifact",
        "mcpb-smoke",
        "seal-candidate",
    }
    stage_text = yaml.safe_dump(stage_jobs)
    assert not re.search(r"(?m)^\s+(contents|packages): write$", stage_text)
    assert "gh release create" not in stage_text
    assert "git push" not in stage_text

    pointer = yaml.safe_dump(workflow["jobs"]["advance-mutable-pointers"])
    assert workflow["jobs"]["advance-mutable-pointers"]["needs"] == "reconcile-public-release"
    assert "gh release edit" in pointer and "--latest" in pointer
    assert "biomcp:latest" in pointer
    assert "merge --ff-only" in pointer


def test_no_other_workflow_exposes_release_publication() -> None:
    routes = ("gh release create", "uv publish", "skopeo copy", "git push")
    for path in WORKFLOWS.glob("*.yml"):
        if path == RELEASE_WORKFLOW:
            continue
        text = path.read_text(encoding="utf-8")
        assert not any(route in text for route in routes), path.name
