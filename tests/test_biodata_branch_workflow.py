from __future__ import annotations

from copy import deepcopy
from pathlib import Path
import re

import pytest
import yaml


ROOT = Path(__file__).resolve().parents[1]
WORKFLOW = ROOT / ".github/workflows/biodata-1.0.yml"
RUNNER = ROOT / "tools/check-biodata-1.0"
BRANCH = "biodata/biomcp-1.0"
TEST_FILES = (
    "tests/test_biodata_boundary.py",
    "tests/test_source_package_boundary.py",
    "tests/test_ctgov_trial_search_detail_reuse.py",
    "tests/test_nci_filter_transport.py",
    "tests/test_biodata_branch_workflow.py",
)
PINNED_ACTIONS = {
    "actions/checkout@11bd71901bbe5b1630ceea73d27597364c9af683",
    "dtolnay/rust-toolchain@4360b52568e2003a75bf9bc1d59f33a8e3fc893c",
    "actions/setup-python@a26af69be951a213d495a4c3e4e4022e16d87065",
    "astral-sh/setup-uv@38f3f104447c67c051c4a08e39b64a148898af3a",
}
PINNED_VERSIONS = {
    "RUST_TOOLCHAIN": "1.93.1",
    "PYTHON_VERSION": "3.12.3",
    "UV_VERSION": "0.8.0",
    "BUBBLEWRAP_VERSION": "0.9.0-1ubuntu0.1",
    "APPARMOR_VERSION": "4.0.1really4.0.1-0ubuntu0.24.04.7",
}
BROAD_OR_EXTERNAL_COMMANDS = (
    "make lint",
    "make test",
    "make spec",
    "make full-feature-check",
    "make release-gate",
    "curl ",
    "wget ",
    "uv publish",
    "cargo publish",
    "git push",
    "deploy",
)


def _action_references(value: object) -> list[str]:
    if isinstance(value, dict):
        references = [str(value["uses"])] if "uses" in value else []
        return references + [
            reference
            for child in value.values()
            for reference in _action_references(child)
        ]
    if isinstance(value, list):
        return [reference for child in value for reference in _action_references(child)]
    return []


def _permission_values(value: object) -> list[object]:
    if isinstance(value, dict):
        return [child for key, child in value.items() if key == "permissions"] + [
            permission
            for child in value.values()
            for permission in _permission_values(child)
        ]
    if isinstance(value, list):
        return [
            permission for child in value for permission in _permission_values(child)
        ]
    return []


def _runner_test_files(runner: str) -> tuple[str, ...]:
    match = re.search(r"readonly TEST_FILES=\(\n(?P<body>.*?)\n\)", runner, re.DOTALL)
    if match is None:
        return ()
    return tuple(re.findall(r'^\s+"([^"]+)"$', match.group("body"), re.MULTILINE))


def _violations(workflow_text: str, runner: str) -> list[str]:
    try:
        workflow = yaml.safe_load(workflow_text)
    except yaml.YAMLError:
        return ["workflow must be valid YAML"]
    if not isinstance(workflow, dict):
        return ["workflow must be a mapping"]

    violations: list[str] = []
    trigger = workflow.get(True, {})
    if trigger != {"push": {"branches": [BRANCH]}}:
        violations.append("workflow must run only for pushes to the dedicated branch")
    if workflow.get("permissions") != {"contents": "read"}:
        violations.append("repository permissions must be read-only")
    if _permission_values(workflow) != [{"contents": "read"}]:
        violations.append("jobs and steps must not widen permissions")
    if workflow.get("env") != PINNED_VERSIONS:
        violations.append("tool versions must remain exact")

    jobs = workflow.get("jobs", {})
    if set(jobs) != {"focused-biodata-verification"}:
        violations.append("workflow must contain one required job")
        return violations
    job = jobs["focused-biodata-verification"]
    if job.get("runs-on") != "ubuntu-24.04" or job.get("timeout-minutes") != 30:
        violations.append("runner and timeout must remain exact")
    if "if" in job or job.get("continue-on-error") is not None:
        violations.append("the focused job must not be skipped or made non-blocking")
    steps = job.get("steps", [])
    if not steps:
        violations.append("the focused job must contain steps")
    for step in steps:
        if "if" in step or step.get("continue-on-error") is not None:
            violations.append("focused steps must not be skipped or made non-blocking")
    actions = set(_action_references(workflow))
    if actions != PINNED_ACTIONS or any(
        re.fullmatch(r"[^@\s]+@[0-9a-f]{40}", action) is None for action in actions
    ):
        violations.append("actions must remain on the accepted immutable revisions")
    checkout = next(
        (
            step
            for step in steps
            if str(step.get("uses", "")).startswith("actions/checkout@")
        ),
        {},
    )
    if checkout.get("with", {}).get("persist-credentials") is not False:
        violations.append("checkout credentials must not persist")

    combined = f"{workflow_text}\n{runner}".lower()
    for command in BROAD_OR_EXTERNAL_COMMANDS:
        if command in combined:
            violations.append(f"forbidden broad or external command: {command}")
    if (
        "secrets." in combined
        or "uses: actions/cache@" in combined
        or "cache:" in combined
    ):
        violations.append("secrets and caches are forbidden")
    if "tools/check-biodata-1.0" not in workflow_text:
        violations.append("workflow must invoke the repository-owned focused runner")
    if "tools/check-biodata-boundary.py" not in runner:
        violations.append("runner must invoke the BioData boundary checker first")
    if "unset NCI_API_KEY" not in runner or "env -u NCI_API_KEY" not in runner:
        violations.append("runner must clear the inherited NCI credential")
    if "tools/run-offline" not in runner:
        violations.append("runner must enter the existing offline sandbox")
    if 'mktemp -d "$ROOT/.cache/biodata-1.0.XXXXXX"' not in runner:
        violations.append("runner temporary storage must stay in the worktree")
    if "cargo build" in runner or "cargo run" in runner:
        violations.append("runner must require an already-built binary")
    if _runner_test_files(runner) != TEST_FILES:
        violations.append("runner must select exactly the five accepted test files")
    return violations


def test_focused_workflow_and_runner_match_the_accepted_contract() -> None:
    workflow_text = WORKFLOW.read_text(encoding="utf-8")
    runner = RUNNER.read_text(encoding="utf-8")
    assert not _violations(workflow_text, runner)
    assert RUNNER.stat().st_mode & 0o111


@pytest.mark.parametrize(
    ("target", "old", "new"),
    (
        ("workflow", "contents: read", "contents: write"),
        ("workflow", "persist-credentials: false", "persist-credentials: true"),
        ("workflow", next(iter(PINNED_ACTIONS)), "actions/checkout@v4"),
        ("workflow", "RUST_TOOLCHAIN: 1.93.1", "RUST_TOOLCHAIN: stable"),
        ("workflow", "timeout-minutes: 30", "timeout-minutes: 45"),
        ("workflow", f"branches: [{BRANCH}]", "branches: [main]"),
        ("workflow", "runs-on: ubuntu-24.04", "if: false\n    runs-on: ubuntu-24.04"),
        (
            "workflow",
            "run: tools/check-biodata-1.0",
            "continue-on-error: true\n        run: tools/check-biodata-1.0",
        ),
        ("runner", "tools/run-offline", "tools/run-online"),
        ("runner", "unset NCI_API_KEY", "true # retain NCI_API_KEY"),
        ("runner", TEST_FILES[2], "tests/test_live_provider.py"),
        ("runner", "uv run --no-sync pytest", "make test && uv run --no-sync pytest"),
        (
            "workflow",
            "uv sync --extra dev --locked",
            "uv sync --extra dev --locked\n          curl https://example.com",
        ),
        (
            "workflow",
            "uv sync --extra dev --locked",
            "uv sync --extra dev --locked\n          uv publish",
        ),
    ),
)
def test_representative_mutations_are_rejected(target: str, old: str, new: str) -> None:
    workflow_text = WORKFLOW.read_text(encoding="utf-8")
    runner = RUNNER.read_text(encoding="utf-8")
    if target == "workflow":
        assert old in workflow_text
        workflow_text = workflow_text.replace(old, new, 1)
    else:
        assert old in runner
        runner = runner.replace(old, new, 1)
    assert _violations(workflow_text, runner)


def test_job_permission_cache_and_deployment_mutations_are_rejected() -> None:
    workflow = yaml.safe_load(WORKFLOW.read_text(encoding="utf-8"))
    runner = RUNNER.read_text(encoding="utf-8")
    job = workflow["jobs"]["focused-biodata-verification"]

    for mutation in (
        {"permissions": {"contents": "write"}},
        {"steps": [*job["steps"], {"uses": f"actions/cache@{'a' * 40}"}]},
        {"steps": [*job["steps"], {"run": "deploy production"}]},
    ):
        changed = deepcopy(workflow)
        changed["jobs"]["focused-biodata-verification"].update(mutation)
        assert _violations(yaml.safe_dump(changed), runner)
