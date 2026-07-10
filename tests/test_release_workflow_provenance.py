from __future__ import annotations

import re
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[1]
RELEASE_WORKFLOW = REPO_ROOT / ".github" / "workflows" / "release.yml"
SOURCE_JOBS = (
    "validate",
    "build",
    "homebrew-tap",
    "docker-publish",
    "pypi-build",
    "deploy-docs",
)
CANONICAL_SHA = "${{ needs.release-ref.outputs.sha }}"


def _workflow() -> str:
    return RELEASE_WORKFLOW.read_text(encoding="utf-8")


def _job(workflow: str, name: str) -> str:
    match = re.search(
        rf"^  {re.escape(name)}:\n(.*?)(?=^  [a-z][a-z0-9-]*:\n|\Z)",
        workflow,
        flags=re.MULTILINE | re.DOTALL,
    )
    assert match is not None, f"missing release job: {name}"
    return match.group(1)


def test_every_source_consuming_release_job_checks_out_canonical_sha() -> None:
    workflow = _workflow()

    for name in SOURCE_JOBS:
        job = _job(workflow, name)
        assert job.count("uses: actions/checkout@v4") == 1, (
            f"{name} must have exactly one auditable source checkout"
        )
        assert f"ref: {CANONICAL_SHA}" in job, f"{name} checkout is not release-pinned"
        assert "release-ref" in job.split("steps:", maxsplit=1)[0], (
            f"{name} must depend on canonical release-ref resolution"
        )


def test_validation_rejects_checkout_that_differs_from_resolved_tag_commit() -> None:
    validate = _job(_workflow(), "validate")
    checkout_end = validate.index("name: Cache protoc")
    provenance_gate = validate[:checkout_end]

    assert "Verify checkout matches requested tag commit" in provenance_gate
    assert 'test "$(git rev-parse HEAD)" = ' in provenance_gate
    assert CANONICAL_SHA in provenance_gate


def test_native_binary_sha_guard_precedes_packaging_and_is_platform_appropriate() -> None:
    build = _job(_workflow(), "build")
    unix_guard = build.index("Verify binary embeds checkout SHA (Unix)")
    windows_guard = build.index("Verify binary embeds checkout SHA (Windows)")
    first_package = build.index("Package (tar.gz)")

    assert unix_guard < first_package
    assert windows_guard < first_package
    assert '${expected:0:8}' in build
    assert '"$bin" version | grep -F "(git $expected,"' in build
    assert 'shell: pwsh' in build[windows_guard:first_package]
    assert '.Substring(0, 8)' in build
    assert '$version = & $bin version' in build
    assert 'throw "Binary SHA mismatch: $version"' in build
    assert "scripts/release-smoke.sh" not in _job(_workflow(), "validate")
