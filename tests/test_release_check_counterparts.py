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
        # Native Windows execution belongs to the complete rehearsal; this
        # contract checks its required command and isolated environment only.
        "Check cold cache statistics in native Windows PowerShell":
            ("tests/test_installed_wheel_smoke.py", "def test_main_and_release_use_the_shared_installed_wheel_smoke("),
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
    "astral-sh/setup-uv", "Swatinem/rust-cache", "docker/build-push-action", "docker/login-action",
    "docker/setup-buildx-action", "docker/setup-qemu-action", "dtolnay/rust-toolchain",
}


# Reusable qualification and composite actions are executable containers too.
# This inventory links steps, not shell semantics: edits inside a mapped shell
# block still require source review and its existing behavioral contracts.
CI_COUNTERPARTS = {
    ("changes", "Decide whether the push touches only docs"): ("scripts/ci-classify-push.sh", "git merge-base"),
    ("build", "Build the dev test archive and binaries"): ("Makefile", "prepare-test:"),
    ("build", "Fetch the offline lanes' cargo registry"): (PROVENANCE, "cargo fetch --locked"),
    ("lint", "Canonical lint gate"): ("Makefile", "lint:"),
    ("rust-tests", "Canonical Rust test gate"): ("Makefile", "test-rust:"),
    ("python-lane", "Fetch the cargo registry for the offline contract tests"): (PROVENANCE, "cargo fetch --locked"),
    ("python-lane", "Canonical Python contract gate"): ("Makefile", "test-python:"),
    ("spec-lane", "Canonical specification gate"): ("Makefile", "spec:"),
    ("stress-lane", "Stress lane"): ("scripts/run-nightly-checks.sh", "make stress"),
    ("release-panic", "Release-mode panic recovery contract"): ("scripts/run-nightly-checks.sh", "cargo nextest run --release --test rmcp_client_contract rmcp_stdio_recovers_from_tool_panic"),
    ("full-features", "make full-feature-check FULL_FEATURE_PROFILE=release"): ("Makefile", "full-feature-check:"),
    ("generated-sources", "scripts/regenerate-alphagenome-proto --check"): ("scripts/run-nightly-checks.sh", "scripts/regenerate-alphagenome-proto --check"),
    ("repository-contracts", "Sync the Python contract environment"): ("Makefile", "sync-python-dev:"),
    ("repository-contracts", "Python contract tests (docs and records)"): ("Makefile", "test-contracts-prepared:"),
    ("repository-contracts", "Documentation build"): ("Makefile", "mkdocs build --strict"),
    ("repository-contracts", "cmp --silent install.sh docs/install.sh"): ("install.sh", "#!/"),
    ("repository-contracts", "bash scripts/check-version-sync.sh"): ("scripts/check-version-sync.sh", "Cargo.toml"),
    ("repository-contracts", "bash scripts/check-no-climb-tracked.sh"): ("scripts/check-no-climb-tracked.sh", "git ls-files"),
    ("release-checks", "Require the changelog to cover the branch"): ("scripts/check-release-metadata.py", "scripts/check-changelog-coverage.py"),
    ("release-checks", "Require the release's green-CI query to run"): ("tests/test_ci_workflow_contract.py", "green-CI"),
}
# Native Windows commands execute in the complete rehearsal, not nightly Linux.
for name in (
    "Compare released Windows cache behavior",
    "Check cache-lock contention and article-search deadlines",
    "Compile Windows CSpec and managed-state boundaries",
    "Check with-build-identity passes exit codes through",
):
    CI_COUNTERPARTS["windows-contracts", name] = ("tests/test_ci_workflow_contract.py", "windows-contracts")
for job in ("rust-tests", "python-lane", "spec-lane", "stress-lane"):
    CI_COUNTERPARTS[job, "Restore the dev binaries' exec bits"] = ("tests/test_ci_workflow_contract.py", "test_the_build_job_uploads_one_dev_build_the_lanes_download")

# Setup dispositions point at the local bootstrap or the existing setup contract.
ACTION_COUNTERPARTS = {
    ("lane-setup", "Install the offline sandbox"): ("tools/run-offline", "bwrap"),
    ("lane-setup", "Install pinned nextest"): ("scripts/install-nextest.sh", "NEXTEST"),
    ("lane-setup", "Install pinned mustmatch"): ("scripts/run-specs.sh", "mustmatch"),
    ("lane-setup", "Install pinned lint tools"): ("tools/bootstrap-lint-tools", "shellcheck"),
    ("lane-setup", "Record exact tool versions"): (".github/actions/lane-setup/action.yml", "command -v"),
}


# Pages writes remain publication-only; strict construction is also local.
DOCS_COUNTERPARTS = {
    ("publish-docs", "Fetch the existing Pages history"): ("tests/test_docs_publication_contract.py", "git fetch origin gh-pages"),
    ("publish-docs", "Install the locked Python environment"): ("Makefile", "sync-python-dev:"),
    ("publish-docs", "Build documentation strictly"): ("Makefile", "mkdocs build --strict"),
    ("publish-docs", "Admit only the selected revision without replacing newer documentation"): ("tests/test_docs_publication_contract.py", "test_docs_admission_retains_newer_revisions_and_refuses_divergence"),
    ("publish-docs", "Publish the validated site to the Pages branch"): ("tests/test_docs_publication_contract.py", "mkdocs gh-deploy --strict --force"),
    ("publish-docs", "Request a Pages build from the configured branch"): ("tests/test_docs_publication_contract.py", "/pages/builds"),
    ("publish-docs", "Verify the exact revision and static documentation are live"): ("scripts/verify-docs-publication.py", "def main("),
}
REUSED_WORKFLOWS = {
    "./.github/workflows/ci.yml": CI_COUNTERPARTS,
    "./.github/workflows/docs-edge.yml": DOCS_COUNTERPARTS,
}


def assert_reused_steps(jobs: dict, root: Path, counterparts: dict, visited: set[str]) -> None:
    for job_name, job in jobs.items():
        assert "uses" not in job, (
            f"{job_name}: {job.get('uses')} has no local/nightly counterpart or platform disposition"
        )
        for step in job.get("steps", []):
            if "run" not in step:
                assert_action_counterpart(step, root, visited)
                continue
            name = step.get("name", step["run"].strip())
            assert (job_name, name) in counterparts, (
                f"{job_name}: {name} has no local/nightly counterpart or platform disposition"
            )
            relative, needle = counterparts[job_name, name]
            assert needle in (root / relative).read_text(), f"{job_name}: {name} lost {relative}: {needle}"


def assert_action_counterpart(step: dict, root: Path, visited: set[str]) -> None:
    action = step.get("uses", "").split("@", 1)[0]
    assert action in SETUP_AND_ARTIFACT_ACTIONS, (
        f"{action} has no local/nightly counterpart or platform disposition"
    )
    if action.startswith("./") and action not in visited:
        visited.add(action)
        document = yaml.safe_load((root / action / "action.yml").read_text())
        assert_reused_steps({"lane-setup": document["runs"]}, root, ACTION_COUNTERPARTS, visited)


def assert_counterparts(workflow: dict, root: Path) -> None:
    visited: set[str] = set()
    for job_name, job in workflow["jobs"].items():
        if "uses" in job:
            assert job["uses"] in REUSED_WORKFLOWS, (
                f"{job_name}: {job['uses']} has no local/nightly counterpart or platform disposition"
            )
            reused = yaml.safe_load((root / job["uses"]).read_text())
            assert_reused_steps(reused["jobs"], root, REUSED_WORKFLOWS[job["uses"]], visited)
        for step in job.get("steps", []):
            if "run" not in step:
                assert_action_counterpart(step, root, visited)
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



@pytest.mark.parametrize("container", ["ci", "action"])
def test_new_check_inside_a_reused_container_needs_a_counterpart(tmp_path: Path, container: str) -> None:
    for child in ROOT.iterdir():
        if child.name != ".github":
            (tmp_path / child.name).symlink_to(child, target_is_directory=child.is_dir())
    shutil.copytree(ROOT / ".github", tmp_path / ".github")
    relative = (".github/workflows/ci.yml" if container == "ci"
                else ".github/actions/lane-setup/action.yml")
    path = tmp_path / relative
    document = yaml.safe_load(path.read_text())
    steps = (document["jobs"]["stress-lane"]["steps"] if container == "ci"
             else document["runs"]["steps"])
    steps.append({"name": "Check a new condition inside reused code", "run": "scripts/new-check"})
    path.write_text(yaml.safe_dump(document))
    release = yaml.safe_load((ROOT / ".github/workflows/release.yml").read_text())
    with pytest.raises(AssertionError, match="has no local/nightly counterpart"):
        assert_counterparts(release, tmp_path)

def test_nightly_reuses_the_existing_release_only_commands() -> None:
    nightly = (ROOT / "scripts/run-nightly-checks.sh").read_text()
    ci = yaml.safe_load((ROOT / ".github/workflows/ci.yml").read_text())
    for job_name in ("stress-lane", "release-panic", "generated-sources"):
        # The final command is the real check, after any build/setup commands.
        step = next(s for s in reversed(ci["jobs"][job_name]["steps"]) if "run" in s)
        for command in step["run"].strip().splitlines():
            assert command.strip() in nightly, f"nightly lost {job_name}: {command}"
    assert "make prepare-test" in nightly
    assert "make lint BIOMCP_ADVISORY_SCOPE=always" in nightly
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
        "make prepare-test", "make lint BIOMCP_ADVISORY_SCOPE=always",
        "make stress", "make test-long",
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
def test_stress_uses_one_cpu_available_to_the_job(tmp_path: Path) -> None:
    # Restrict the recipe to one allowed CPU, normally outside CPU zero.
    selected = max(os.sched_getaffinity(0))
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
        shutil.which("taskset"), "-c", str(selected),
        shutil.which("make"), "-f", str(ROOT / "Makefile"), "stress", "MAKE=" + str(commands / "make"),
        "SPEC_BIN=" + str(binary), "BIOMCP_STRESS_REPEAT=1",
    ], cwd=tmp_path, env={
        **os.environ, "PATH": str(commands) + os.pathsep + os.environ["PATH"],
        "CALLS": str(calls),
    }, capture_output=True, text=True)
    assert result.returncode == 0, result.stdout + result.stderr
    expected = "-c " + str(selected)
    assert calls.read_text().splitlines() == [expected, expected]
