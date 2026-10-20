"""Keep release checks connected to earlier execution or a platform boundary.

Nightly Linux runs the local installed-wheel smoke. Other native platforms and
publication boundaries retain their executable contracts and full rehearsal.
Adding a release command requires choosing its earlier command or explaining
that boundary here; merely retaining a familiar job name does not admit it.
"""

from __future__ import annotations

from copy import deepcopy
import os
from pathlib import Path
import shutil
import subprocess

import pytest
import yaml


ROOT = Path(__file__).resolve().parents[1]
PROVENANCE = "tests/test_release_workflow_provenance.py"

# Each value names an actual local implementation or executable contract.
# Setup, publication and native-artifact dispositions retain the full rehearsal;
# they do not claim that a Linux source test proves another platform's artifact.
COUNTERPARTS = {
    "version-check": {
        "Require the tag and committed versions to agree":
            ("scripts/check-release-metadata.py", '"scripts/check-release-versions.py"'),
        "Require the changelog to cover merged tickets":
            ("scripts/check-release-metadata.py", '"scripts/check-changelog-coverage.py"'),
    },
    "release-gate": {
        "Fetch the cargo registry for the offline marker set":
            (PROVENANCE, "def test_the_release_gate_marker_job_and_make_run_the_same_selection("),
        "Run the release-gate marker set": ("Makefile", "release-gate-markers:"),
    },
    "create-draft": {
        "Render the release notes from the changelog":
            ("release/release_notes.py", "def main("),
        "Create the draft release":
            ("tests/test_release_draft_notes.py", "def test_"),
    },
    "build": {
        "Build release binary": (PROVENANCE, "def test_release_builders_use_admitted_version_and_actual_revision("),
        "Build and package inside the manylinux 2_28 container":
            (PROVENANCE, "def test_linux_floor_contract("),
        "Package (macOS)": (PROVENANCE, "def test_pipeline_jobs_contract("),
        "Package (Windows)": (PROVENANCE, "def test_pipeline_jobs_contract("),
    },
    "publish-assets": {
        "Upload release assets": (PROVENANCE, "def test_pipeline_jobs_contract("),
    },
    "pypi-build": {
        "Install pinned maturin": (PROVENANCE, "def test_pipeline_jobs_contract("),
        "Build wheels": (PROVENANCE, "def test_release_builders_use_admitted_version_and_actual_revision("),
        "Build wheels inside the manylinux 2_28 container":
            (PROVENANCE, "def test_linux_floor_contract("),
    },
    "wheel-smoke": {
        "Install the wheel into a clean venv":
            (PROVENANCE, "def test_pipeline_jobs_contract("),
        "Run the wheel inside the manylinux 2_28 container":
            ("scripts/check-linux-wheel.sh", "docker run"),
        "Smoke the installed wheel on every shipped platform":
            ("scripts/installed-wheel-smoke.sh", "--version"),
    },
    "docs-live": {
        "Resolve the tag commit": (PROVENANCE, "def test_pipeline_jobs_contract("),
        "Require the live documentation revision to equal or descend from the tag":
            (PROVENANCE, "def test_pipeline_jobs_contract("),
    },
    "pypi-publish": {
        "Publish to PyPI with trusted publishing (production only)":
            (PROVENANCE, "def test_pypi_publish_guard_runs_the_right_publisher_under_each_repository("),
    },
    "homebrew-tap": {
        "Download checksums from the release":
            (PROVENANCE, "def test_homebrew_guard_renders_and_checks_both_checksums_then_guards_the_push("),
        "Update Homebrew tap (production only)":
            (PROVENANCE, "def test_homebrew_guard_renders_and_checks_both_checksums_then_guards_the_push("),
    },
    "container-publish": {
        "Resolve the image version and source revision":
            (PROVENANCE, "def test_pipeline_jobs_contract("),
        "Stage the released Linux executables":
            (PROVENANCE, "def test_pipeline_jobs_contract("),
        "Smoke the linux/amd64 image from the registry":
            (PROVENANCE, "def test_pipeline_jobs_contract("),
        "Smoke the linux/arm64 image from the registry":
            (PROVENANCE, "def test_pipeline_jobs_contract("),
    },
    "publish-release": {
        "Move latest after both platform smokes pass":
            (PROVENANCE, "def test_the_latest_step_fails_when_gh_fails("),
        "Publish the GitHub release after every publisher succeeds":
            (PROVENANCE, "def test_pipeline_jobs_contract("),
    },
    "rehearsal-docs": {
        "Validate the candidate documentation without deployment":
            (PROVENANCE, "def test_rehearsal_builds_have_no_publication_permissions_or_credentials("),
    },
    "rehearsal-container": {
        "Stage the checked Linux executables":
            (PROVENANCE, "def test_pipeline_jobs_contract("),
        "Smoke the local platform image and retain it":
            (PROVENANCE, "def test_pipeline_jobs_contract("),
    },
    "rehearsal-payloads": {
        "Validate release publication payloads without credentials":
            (PROVENANCE, "def test_pipeline_jobs_contract("),
    },
    "rehearsal-complete": {
        "Record the limits of a successful rehearsal":
            (PROVENANCE, "def test_pipeline_jobs_contract("),
    },
}

# These commands have an earlier local caller. Other rows describe a native
# artifact or publication boundary: their executable contracts run routinely,
# and the complete nonpublishing rehearsal still executes the real platform
# checks before a tag. Neither a source test nor nightly Linux replaces that.
LOCAL_COMMANDS = {
    ("version-check", "Require the tag and committed versions to agree"):
        "scripts/check-release-versions.py",
    ("version-check", "Require the changelog to cover merged tickets"):
        "scripts/check-changelog-coverage.py",
    ("release-gate", "Run the release-gate marker set"): "make release-gate-markers",
    ("create-draft", "Render the release notes from the changelog"):
        "release/release_notes.py",
    ("wheel-smoke", "Smoke the installed wheel on every shipped platform"):
        "scripts/installed-wheel-smoke.sh",
}

# These actions prepare execution or transport/publish the already checked
# artifacts. A new checking action needs a counterpart, just like a run step.
SETUP_AND_ARTIFACT_ACTIONS = {
    "./.github/actions/lane-setup", "actions/checkout", "actions/download-artifact",
    "actions/setup-python", "actions/upload-artifact", "arduino/setup-protoc",
    "astral-sh/setup-uv", "docker/build-push-action", "docker/login-action",
    "docker/setup-buildx-action", "docker/setup-qemu-action", "dtolnay/rust-toolchain",
}


def assert_counterparts(workflow: dict, root: Path) -> None:
    for job_name, job in workflow["jobs"].items():
        for step in job.get("steps", []):
            if "run" not in step:
                action = step.get("uses", "").split("@", 1)[0]
                assert action in SETUP_AND_ARTIFACT_ACTIONS, (
                    f"{job_name}: {action} has no local/nightly counterpart or platform disposition"
                )
                continue
            name = step.get("name", "<unnamed>")
            assert name in COUNTERPARTS.get(job_name, {}), (
                f"{job_name}: {name} has no local/nightly counterpart or platform disposition"
            )
            relative, command = COUNTERPARTS[job_name][name]
            source = root / relative
            assert source.is_file(), f"{job_name}: {name} lost counterpart {relative}"
            assert command in source.read_text(), f"{job_name}: {name} lost {command}"
            if (job_name, name) in LOCAL_COMMANDS:
                assert LOCAL_COMMANDS[job_name, name] in step["run"], (
                    f"{job_name}: {name} no longer runs its local counterpart"
                )


def test_release_commands_have_local_or_platform_counterparts() -> None:
    release = yaml.safe_load((ROOT / ".github/workflows/release.yml").read_text())
    assert_counterparts(release, ROOT)


@pytest.mark.parametrize("step", [
    {"name": "Check a new release-only condition", "run": "scripts/check-new-release-condition"},
    {"name": "Check a new release-only condition", "uses": "example/new-release-check@abc"},
])
def test_unmapped_release_check_fails_even_inside_an_existing_job(step: dict) -> None:
    release = yaml.safe_load((ROOT / ".github/workflows/release.yml").read_text())
    planted = deepcopy(release)
    planted["jobs"]["release-gate"]["steps"].append(step)
    with pytest.raises(AssertionError, match="has no local/nightly counterpart"):
        assert_counterparts(planted, ROOT)


def test_nightly_reuses_the_existing_release_only_commands() -> None:
    nightly = (ROOT / "scripts/run-nightly-checks.sh").read_text()
    ci = yaml.safe_load((ROOT / ".github/workflows/ci.yml").read_text())
    for job_name in ("stress-lane", "release-panic", "generated-sources"):
        # The final command is the real check, after any build/setup commands.
        step = next(s for s in reversed(ci["jobs"][job_name]["steps"]) if "run" in s)
        for command in step["run"].strip().splitlines():
            assert command.strip() in nightly, f"nightly lost {job_name}: {command}"
    assert "make prepare-test" in nightly
    assert "make test-long" in nightly
    assert "make feature-coverage" in nightly
    for target in ("repo-audit", "full-feature-check", "installed-wheel-smoke"):
        assert f"make {target}" in nightly
    assert "nightly-checks:" in (ROOT / "Makefile").read_text()


@pytest.mark.parametrize("failure", ["", "stress", "installed-wheel-smoke"])
def test_nightly_executes_the_checks_and_stops_at_a_failure(tmp_path: Path, failure: str) -> None:
    scripts = tmp_path / "scripts"
    scripts.mkdir()
    (tmp_path / "tools").mkdir()
    runner = scripts / "run-nightly-checks.sh"
    runner.write_text((ROOT / "scripts/run-nightly-checks.sh").read_text())
    commands = tmp_path / "commands"
    commands.mkdir()
    for path, body in {
        commands / "make": 'echo "make $*" >> "$CALLS"; if [ "$*" = "$FAILURE" ]; then exit 19; fi',
        commands / "cargo": 'echo "cargo $*" >> "$CALLS"',
        tmp_path / "tools/with-build-identity": 'exec "$@"',
        scripts / "regenerate-alphagenome-proto": 'echo "proto $*" >> "$CALLS"',
    }.items():
        path.write_text('#!/bin/sh\n' + body + '\n')
        path.chmod(0o755)
    calls = tmp_path / "calls"
    result = subprocess.run(["bash", str(runner)], env={
        **os.environ, "PATH": str(commands) + os.pathsep + os.environ["PATH"],
        "CALLS": str(calls), "FAILURE": failure,
    }, capture_output=True, text=True)
    expected = [
        "make prepare-test", "make stress", "make test-long",
        "cargo build --release --locked",
        "cargo nextest run --release --test rmcp_client_contract rmcp_stdio_recovers_from_tool_panic",
        "proto --check",
        "make repo-audit", "make full-feature-check", "make installed-wheel-smoke",
        "make feature-coverage",
    ]
    assert result.returncode == (19 if failure else 0), result.stderr
    if failure:
        expected = expected[:expected.index("make " + failure) + 1]
    assert calls.read_text().splitlines() == expected


@pytest.mark.skipif(not hasattr(os, "sched_getaffinity"), reason="Linux stress affinity")
def test_stress_uses_two_cpus_available_to_the_job(tmp_path: Path) -> None:
    available = sorted(os.sched_getaffinity(0))
    if len(available) < 2:
        pytest.skip("stress needs two available CPUs")
    (tmp_path / ".cache").mkdir()
    (tmp_path / ".cache/routine-tests.tar.zst").write_text("prepared archive")
    binary = tmp_path / "biomcp"
    binary.touch(mode=0o755)
    commands = tmp_path / "commands"
    commands.mkdir()
    for name, body in {
        "taskset": 'echo "$1 $2" >> "$CALLS"',
        "make": "exit 0",
    }.items():
        path = commands / name
        path.write_text("#!/bin/sh\n" + body + "\n")
        path.chmod(0o755)
    calls = tmp_path / "calls"
    result = subprocess.run([
        shutil.which("make"), "-f", str(ROOT / "Makefile"), "stress", "MAKE=" + str(commands / "make"),
        "SPEC_BIN=" + str(binary), "BIOMCP_STRESS_REPEAT=1",
    ], cwd=tmp_path, env={
        **os.environ, "PATH": str(commands) + os.pathsep + os.environ["PATH"],
        "CALLS": str(calls),
    }, capture_output=True, text=True)
    assert result.returncode == 0, result.stdout + result.stderr
    expected = "-c " + ",".join(map(str, available[:2]))
    assert calls.read_text().splitlines() == [expected, expected]
