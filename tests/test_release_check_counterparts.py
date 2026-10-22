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
import sys
import xml.etree.ElementTree as ET

import pytest
import yaml

from make_fixture import make_fixture_env
from release_workflow_inventory import (
    COUNTERPARTS, LOCAL_COMMANDS, assert_step_inventory,
    release_inventory_admission as release_inventory_admission,
    release_inventory_state as release_inventory_state,
)


ROOT = Path(__file__).resolve().parents[1]
PROVENANCE = "tests/test_release_workflow_provenance.py"

# The shared release inventory assigns each run step its earlier executable
# contract or its native-artifact/publication boundary. Rehearsal still executes
# those real platform checks before a tag.

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
    ("release-panic", "Release-mode panic recovery contract"): ("scripts/run-nightly-checks.sh", "cargo nextest run --release --lib rmcp_stdio_recovers_from_tool_panic"),
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


ACTION_COUNTERPARTS["lane-setup", "Read declared Rust toolchain"] = ("tools/rust-toolchain-version", "rust-toolchain.toml")

# Toolchain setup reads the same declaration used by local Cargo commands.
CI_COUNTERPARTS["windows-contracts", "Read declared Rust toolchain"] = ("tools/rust-toolchain-version", "rust-toolchain.toml")

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
    assert_step_inventory(release)
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
    result = subprocess.run(["bash", str(runner)], env=make_fixture_env(
        tmp_path, {"CALLS": str(calls), "FAILURE": failure}, commands=(commands,),
    ), capture_output=True, text=True)
    expected = [
        "make prepare-test", "make lint BIOMCP_ADVISORY_SCOPE=always",
        "make stress", "make test-long",
        "cargo build --release --locked",
        "cargo nextest run --release --lib rmcp_stdio_recovers_from_tool_panic",
        "proto --check",
        "make repo-audit", "make full-feature-check", "make installed-wheel-smoke",
        "make feature-coverage",
    ]
    assert result.returncode == (19 if failure else 0), result.stderr
    if failure:
        expected = expected[:expected.index("make " + failure) + 1]
    assert calls.read_text().splitlines() == expected


@pytest.mark.skipif(not hasattr(os, "sched_getaffinity"), reason="Linux stress affinity")
def test_stress_uses_one_cpu_available_to_the_job(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch,
) -> None:
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
    # GNU Make propagates a caller's command-line PATH through MAKEFLAGS.
    # It must not outrank the fixture's fake taskset in the child Make process.
    host_commands = tmp_path / "host-commands"
    host_commands.mkdir()
    host_taskset = host_commands / "taskset"
    host_taskset.write_text('#!/bin/sh\necho inherited-taskset >> "$CALLS"\nexit 97\n')
    host_taskset.chmod(0o755)
    inherited_path = str(host_commands) + os.pathsep + os.environ["PATH"]
    assignment = "PATH=" + inherited_path.replace(" ", "\\ ")
    monkeypatch.setenv("MAKEFLAGS", " -- " + assignment)
    monkeypatch.setenv("MAKEOVERRIDES", assignment)
    monkeypatch.setenv("MAKELEVEL", "7")
    calls = tmp_path / "calls"
    result = subprocess.run([
        shutil.which("taskset"), "-c", str(selected),
        shutil.which("make"), "-f", str(ROOT / "Makefile"), "stress", "MAKE=" + str(commands / "make"),
        "SPEC_BIN=" + str(binary), "BIOMCP_STRESS_REPEAT=1",
    ], cwd=tmp_path, env=make_fixture_env(
        tmp_path, {"CALLS": str(calls)}, commands=(commands,),
    ), capture_output=True, text=True)
    assert result.returncode == 0, result.stdout + result.stderr
    expected = "-c " + str(selected)
    assert calls.read_text().splitlines() == [expected, expected]


@pytest.mark.parametrize("mutation,selection", [
    ("new-step", "both"), ("new-step", "provenance"),
    ("if", "both"), ("shell", "both"), ("body", "both"), ("counterpart", "both"),
])
def test_release_inventory_admission_fails_once_without_hiding_semantic_changes(
    tmp_path: Path, mutation: str, selection: str,
) -> None:
    """Structural drift has one owner; same-identity safety failures never skip."""
    for child in ROOT.iterdir():
        if child.name not in {".github", "tests"}:
            (tmp_path / child.name).symlink_to(child, target_is_directory=child.is_dir())
    shutil.copytree(ROOT / ".github", tmp_path / ".github")
    tests = tmp_path / "tests"
    tests.mkdir()
    for name in ("test_release_check_counterparts.py", "test_release_workflow_provenance.py",
                 "release_workflow_inventory.py"):
        source = ROOT / "tests" / name
        if source.exists():
            shutil.copy(source, tests / name)
    for source in (ROOT / "tests").iterdir():
        destination = tests / source.name
        if not destination.exists():
            destination.symlink_to(source, target_is_directory=source.is_dir())
    workflow = tmp_path / ".github/workflows/release.yml"
    document = yaml.safe_load(workflow.read_text())
    if mutation == "new-step":
        document["jobs"]["release-gate"]["steps"].append(
            {"name": "Check a new release condition", "run": "echo new-check"},
        )
    elif mutation == "counterpart":
        inventory = tests / "release_workflow_inventory.py"
        inventory.write_text(inventory.read_text().replace(
            "tests/test_release_draft_notes.py", "tests/absent-counterpart.py",
        ))
    else:
        step = next(step for step in document["jobs"]["pypi-build"]["steps"]
                    if step.get("name") == "Build wheels")
        key, value = {"if": ("if", "false"), "shell": ("shell", "sh"),
                      "body": ("run", "true")}[mutation]
        step[key] = value
    workflow.write_text(yaml.safe_dump(document))
    report = tmp_path / "report.xml"
    selected = [str(tests / "test_release_workflow_provenance.py") + "::test_pipeline_jobs_contract"]
    if selection == "both":
        selected.insert(0, str(tests / "test_release_check_counterparts.py")
                        + "::test_release_commands_have_local_or_platform_counterparts")
    result = subprocess.run([
        sys.executable, "-m", "pytest", "-q", "-o", "addopts=", "--confcutdir", str(tests),
        "--junitxml", str(report), *selected,
    ], cwd=tmp_path, text=True, capture_output=True)
    assert result.returncode == 1, result.stdout + result.stderr
    failures = ET.parse(report).findall(".//failure")
    assert len(failures) == 1, result.stdout + result.stderr
    if mutation == "new-step":
        assert "tests/release_workflow_inventory.py" in failures[0].text
    else:
        expected = {"if": "not in the allowlist", "shell": "custom shell",
                    "body": "step text drifted", "counterpart": "lost counterpart"}
        assert expected[mutation] in failures[0].text
    expected_skips = 1 if mutation == "new-step" and selection == "both" else 0
    assert len(ET.parse(report).findall(".//skipped")) == expected_skips
