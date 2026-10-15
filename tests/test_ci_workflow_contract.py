"""CI workflow contracts (2026-09-29 review).

The review of the cache round found the docs-only skip unsound
(decided by commit message, skippable by a mislabeled push, and
skipping the docs tests docs-only commits need) and the cache
placement wrong (before checkout and before the toolchain install,
unpinned, without workspace-crate caching). These tests pin the
corrected shapes so drift fails loudly.
"""

from __future__ import annotations

import re
from pathlib import Path

import pytest
import yaml

REPO_ROOT = Path(__file__).resolve().parents[1]
WORKFLOW = REPO_ROOT / ".github" / "workflows" / "ci.yml"
WORKFLOWS = REPO_ROOT / ".github" / "workflows"
ACTION = REPO_ROOT / ".github" / "actions" / "lane-setup" / "action.yml"
DOC = yaml.safe_load(WORKFLOW.read_text(encoding="utf-8"))
ACTION_DOC = yaml.safe_load(ACTION.read_text(encoding="utf-8"))

# Every job that drives the Rust toolchain. All of them wait on the
# changes job for the docs-only classification; the main-only four
# additionally restrict themselves to pushes to main (2026-10-14
# decision: branches trade those jobs for a fast lane, and every
# release tag still needs their green run on its commit).
RUST_JOBS = [
    "build",
    "lint",
    "rust-tests",
    "python-lane",
    "spec-lane",
    "stress-lane",
    "release-panic",
    "full-features",
    "generated-sources",
    "windows-contracts",
]
# The four routine lanes share one build job's dev build (ticket
# 2075): none of them waits on another lane, so they run in
# parallel once the build job uploads its artifact.
LANE_JOBS = ["lint", "rust-tests", "python-lane", "spec-lane"]
MAIN_ONLY_JOBS = ["stress-lane", "release-panic", "full-features", "windows-contracts"]
CACHE_PIN = "Swatinem/rust-cache@6323deb102c322ba6fcbdcafc7e3dddab59af2b6"


def _job(jid: str) -> dict:
    return DOC["jobs"][jid]


def _step_index(jid: str, needle: str) -> int:
    steps = _job(jid)["steps"]
    for i, step in enumerate(steps):
        if needle in str(step.get("name", "")) or needle in str(step.get("uses", "")):
            return i
    raise AssertionError(f"{needle} not found in {jid}")


def test_the_skip_rule_reads_the_changed_files_never_the_message() -> None:
    text = WORKFLOW.read_text(encoding="utf-8")
    assert "head_commit.message" not in text, (
        "the docs-only skip must never read the commit message"
    )
    classify = (REPO_ROOT / "scripts" / "ci-classify-push.sh").read_text(encoding="utf-8")
    # The base is the merge-base with origin/main (a failed tip
    # followed by a markdown commit must not skip the Rust jobs),
    # and executable markdown (spec/, skills/, src/, the compiled
    # CLI reference) never counts as docs.
    assert "git merge-base" in classify
    assert "sdlc/*|notes/*" in classify
    allow_list = classify.split("case")[1].split("esac")[0]
    stripped = allow_list.replace("sdlc/*|notes/*|.github/*.md)", "")
    for executable in ("docs/*", "README.md|", "|README"):
        assert executable not in stripped, (
            "docs/ and README.md are read by Rust tests and compiled in; they must run full CI"
        )
    # CHANGELOG.md left the allow list: the changelog refresh test is
    # needs_binary, so only the canonical lane runs it and a
    # changelog-only push must classify as full CI.
    assert "CHANGELOG.md" not in allow_list, (
        "CHANGELOG.md is read by tests/test_docs_changelog_refresh.py; "
        "a changelog-only push must run the lane that carries it"
    )
    # AGENTS.md left the allow list too (2026-10-14 amendment):
    # spec/surface/mcp.md reads it, so an AGENTS.md-only push must
    # run the spec pages — its docs-only skip is what turned main
    # red on 2026-10-14 until ec4029345.
    allow_line = next(line for line in classify.splitlines() if "sdlc/*|notes/*" in line)
    assert "AGENTS.md" not in allow_line, (
        "AGENTS.md is read by spec/surface/mcp.md; an AGENTS.md-only push "
        "must run the spec pages, so it must not be docs-only"
    )


MOVING_RUNNER_LABELS = ("ubuntu-latest", "macos-latest", "windows-latest")


def _assert_no_moving_runner_label(text: str) -> None:
    for label in MOVING_RUNNER_LABELS:
        assert label not in text, (
            f"{label} is a moving label: GitHub retargets it to a new "
            "image on its own schedule, which can change a release "
            "without a repository change. Pin the image version "
            "instead (ubuntu-24.04, macos-15, windows-2022)."
        )


def test_no_workflow_job_runs_on_a_moving_runner_label() -> None:
    """Ticket 2024: every workflow job runs on a pinned image.

    The 2026-10-07 review found four release and contract jobs still
    on ubuntu-latest while every other job pinned ubuntu-24.04; the
    reopening also pinned the remaining macos-latest and
    windows-latest matrix legs to macos-15 and windows-2022. The ban
    covers the file text, not only runs-on keys, so a matrix or a
    reuse block cannot smuggle a label back in either. The shared
    lane-setup action (ticket 2075) joined the ban: it runs inside
    every lane, so a moving label there moves every lane.
    """
    texts = [path.read_text(encoding="utf-8") for path in sorted(WORKFLOWS.glob("*.yml"))]
    texts.append(ACTION.read_text(encoding="utf-8"))
    for text in texts:
        _assert_no_moving_runner_label(text)


@pytest.mark.parametrize("label", MOVING_RUNNER_LABELS)
def test_the_label_ban_catches_a_planted_moving_label(label: str) -> None:
    """Red proof: planting any moving label anywhere fails the ban."""
    with pytest.raises(AssertionError, match="moving label"):
        _assert_no_moving_runner_label(f"    runs-on: {label}\n")


def _assert_windows_wrapper_step(doc: dict) -> None:
    """Ticket 2024 reopening: the Windows CI job must run
    tools/with-build-identity and fail unless the child's exit code
    propagates. os.execvpe exited 0 at once on Windows while the
    child kept running, so a build step passed with no binary.
    """
    steps = doc["jobs"]["windows-contracts"]["steps"]
    wrapper_steps = [
        step for step in steps if "with-build-identity" in str(step.get("run", ""))
    ]
    assert len(wrapper_steps) == 1, (
        "windows-contracts must exercise tools/with-build-identity on "
        "windows-2022; before this step no job ever ran the script there"
    )
    step = wrapper_steps[0]
    assert step.get("shell") == "bash"
    run = step["run"]
    assert "with-build-identity cargo --version" in run, (
        "the wrapper step must run a trivial cargo command to a green exit"
    )
    # A failing child must surface as a nonzero wrapper exit: the
    # captured status is compared against 0, and the branch fails.
    assert '|| status=$?' in run and 'if [ "$status" -eq 0 ]; then' in run, (
        "the wrapper step must fail when a failing child exits 0 through "
        "the wrapper (the os.execvpe behavior on Windows)"
    )
    assert "raise SystemExit(42)" in run and '[ "$status" -ne 42 ]; then' in run, (
        "the wrapper step must assert an exact exit code passes through, "
        "not merely a nonzero one"
    )


def test_windows_contracts_runs_the_build_identity_wrapper() -> None:
    _assert_windows_wrapper_step(DOC)


def test_the_windows_wrapper_contract_catches_a_dropped_step() -> None:
    """Red proof: removing the wrapper step from the Windows job
    fails the contract."""
    import copy

    doc = copy.deepcopy(DOC)
    doc["jobs"]["windows-contracts"]["steps"] = [
        step
        for step in doc["jobs"]["windows-contracts"]["steps"]
        if "with-build-identity" not in str(step.get("run", ""))
    ]
    with pytest.raises(AssertionError, match="must exercise"):
        _assert_windows_wrapper_step(doc)


def test_every_rust_job_waits_on_the_changes_job() -> None:
    for jid in RUST_JOBS:
        job = _job(jid)
        needs = job.get("needs")
        needs = [needs] if isinstance(needs, str) else needs
        assert "changes" in needs, f"{jid} must need the changes job"
        assert "needs.changes.outputs.docs_only != 'true'" in str(job.get("if", "")), (
            f"{jid} must skip only docs-only pushes"
        )


def test_the_routine_lanes_share_one_build_job_and_run_in_parallel() -> None:
    """Ticket 2075: a build job runs `make prepare-test` once, and the
    four routine lanes download its artifact. No lane waits on
    another lane, so the run's wall time is the build plus the
    longest lane instead of their sum."""
    for jid in (*LANE_JOBS, "stress-lane"):
        needs = _job(jid).get("needs")
        assert needs == ["changes", "build"], (
            f"{jid} must need exactly the changes and build jobs so it runs "
            "in parallel with its sibling lanes"
        )


def test_the_main_only_jobs_skip_every_branch_push() -> None:
    """Ticket 2075: full-features, windows-contracts, stress-lane and
    release-panic run only on pushes to main. A branch's green CI run
    is its merge condition; the release workflow's needs-green-CI
    check on the tagged commit still gates every release on these
    jobs, and a ticket whose changes only they build runs
    `make full-feature-check` itself before landing (2026-10-14
    decision)."""
    for jid in MAIN_ONLY_JOBS:
        if_expr = str(_job(jid).get("if", ""))
        assert "github.event_name == 'push'" in if_expr, (
            f"{jid} must not run on pull requests"
        )
        assert "github.ref == 'refs/heads/main'" in if_expr, (
            f"{jid} must run only on pushes to main"
        )
        assert "needs.changes.outputs.docs_only != 'true'" in if_expr, (
            f"{jid} must still skip docs-only pushes to main"
        )


def test_the_changes_job_covers_the_whole_push() -> None:
    changes = _job("changes")
    assert changes["outputs"]["docs_only"] == "${{ steps.classify.outputs.docs_only }}"
    run_step = next(s for s in changes["steps"] if s.get("id") == "classify")
    assert run_step["run"] == "scripts/ci-classify-push.sh"
    assert run_step["env"]["PUSH_BASE_REF"] == "origin/main"
    assert run_step["env"]["PUSH_AFTER"] == "${{ github.sha }}"


def test_the_changes_job_can_ask_for_the_previous_tips_run() -> None:
    """Ticket 2086: the green-tip route needs the push's before sha
    and a token that may read actions. A job-level permissions block
    replaces the workflow-level one entirely, so the job restates
    contents: read (the checkout needs it) and adds actions: read.
    """
    changes = _job("changes")
    assert changes["permissions"] == {"contents": "read", "actions": "read"}, (
        "the classify job must hold contents: read (checkout) and "
        "actions: read (the run query); a job block that named only "
        "actions would strip the checkout's contents: read"
    )
    run_step = next(s for s in changes["steps"] if s.get("id") == "classify")
    assert run_step["env"]["PUSH_BEFORE"] == "${{ github.event.before }}"
    assert run_step["env"]["GH_TOKEN"] == "${{ github.token }}"


def test_the_cache_follows_checkout_and_toolchain_and_is_pinned() -> None:
    """The rust cache lives in the shared lane-setup action (ticket
    2075). It must run after the lane's checkout and after the pinned
    toolchain install, stay pinned, and cache the workspace crates.
    windows-contracts keeps its own steps: the action installs the
    linux-only offline sandbox packages."""
    action_steps = ACTION_DOC["runs"]["steps"]
    names = [str(step.get("name", "")) for step in action_steps]
    cache = names.index("Cache the Rust build")
    assert cache > names.index("Install pinned Rust"), (
        "the cache must run after the pinned toolchain so its key tracks "
        "the pinned rustc, not the runner default"
    )
    cache_step = action_steps[cache]
    assert cache_step["uses"] == CACHE_PIN, "the cache must be pinned"
    assert cache_step["with"]["cache-workspace-crates"] is True, (
        "the workspace crate must be cached or every run rebuilds it"
    )
    for jid in RUST_JOBS:
        if jid == "windows-contracts":
            checkout = _step_index(jid, "actions/checkout")
            toolchain = _step_index(jid, "rust-toolchain")
            cache = _step_index(jid, "Cache the Rust build")
            assert cache > checkout and cache > toolchain
            cache_step = _job(jid)["steps"][cache]
            assert cache_step["uses"] == CACHE_PIN
            assert cache_step["with"]["cache-workspace-crates"] is True
            continue
        setup = _step_index(jid, ".github/actions/lane-setup")
        assert setup > _step_index(jid, "actions/checkout"), (
            f"{jid}: the lane setup must run after checkout"
        )


def test_the_build_job_uploads_one_dev_build_the_lanes_download() -> None:
    """Ticket 2075: the build job runs `make prepare-test` once and
    uploads the nextest archive, the dev binary and the MCP example
    binary as one artifact; every lane that needs them downloads that
    same artifact instead of rebuilding."""
    steps = _job("build")["steps"]
    build_run = next(s for s in steps if s.get("run") == "make prepare-test")
    upload = next(
        s
        for s in steps
        if str(s.get("uses", "")).startswith("actions/upload-artifact@")
    )
    assert steps.index(upload) > steps.index(build_run), (
        "the artifact must upload after the dev build"
    )
    assert upload["with"]["if-no-files-found"] == "error"
    assert upload["with"]["retention-days"] == 1
    assert upload["with"]["path"].split() == [
        ".cache/routine-tests.tar.zst",
        "target/debug/biomcp",
        "target/debug/examples/rmcp_streamable_http_contract",
    ]
    artifact = upload["with"]["name"]
    for jid in (*LANE_JOBS, "stress-lane"):
        downloads = [
            s
            for s in _job(jid)["steps"]
            if str(s.get("uses", "")).startswith("actions/download-artifact@")
        ]
        assert len(downloads) == 1, f"{jid} must download the dev build once"
        assert downloads[0]["with"]["name"] == artifact
    gates = {
        "build": "make prepare-test",
        "lint": "make lint",
        "rust-tests": "make test-rust",
        "python-lane": "make test-python",
        "spec-lane": "make spec",
        "stress-lane": "make stress",
        "full-features": "make full-feature-check",
    }
    for jid, gate in gates.items():
        runs = [s.get("run") for s in _job(jid)["steps"]]
        assert gate in runs, f"{jid} must run {gate} so CI never copies the gate"


def test_the_lane_setup_action_pins_match_the_workflow_environment() -> None:
    """The action hardcodes the toolchain versions the workflow pins
    in its env block (the uses-with context cannot read the calling
    workflow's env), so the two must agree."""
    text = ACTION.read_text(encoding="utf-8")
    assert f"toolchain: {DOC['env']['RUST_TOOLCHAIN']}" in text
    assert f"python-version: {DOC['env']['PYTHON_VERSION']}" in text
    assert f"version: {DOC['env']['UV_VERSION']}" in text
    assert "@v" not in text and "@stable" not in text


def test_repository_contracts_always_runs_and_tests_the_docs() -> None:
    job = _job("repository-contracts")
    assert "if" not in job, "repository-contracts must never be skipped"
    assert "needs" not in job
    steps = job.get("steps", [])
    runs = [str(s.get("run", "")) for s in steps]
    assert any("pytest" in r and "not needs_binary" in r for r in runs), (
        "docs-only pushes must still run the docs and record tests"
    )
    assert any("mkdocs build --strict" in r for r in runs), (
        "docs-only pushes must still build the documentation"
    )
    assert any("check-test-wait-ratchet.py" in r for r in runs), (
        "the wait ratchet reads sdlc tickets, which docs-only pushes "
        "change; it must run in this always-on job"
    )


def test_nextest_installs_through_the_checksummed_script() -> None:
    for path in (WORKFLOW, ACTION):
        text = path.read_text(encoding="utf-8")
        assert "get.nexte.st" not in text, "the pipe-to-shell installer must be gone"
    assert "scripts/install-nextest.sh" in ACTION.read_text(encoding="utf-8"), (
        "the lane-setup action must install nextest through the checksummed script"
    )
    sha = DOC["env"]["CARGO_NEXTEST_SHA256"]
    assert re.fullmatch(r"[0-9a-f]{64}", sha), "the nextest checksum must be a real sha256"
    script = (REPO_ROOT / "scripts" / "install-nextest.sh").read_text(encoding="utf-8")
    assert "sha256sum --check --status" in script
    assert "CARGO_NEXTEST_SHA256" in script


# Modules that shell out to real cargo without naming a binary
# path (joined literals, plain argv); the 2026-09-29 review found
# three that had slipped past the path scan. New ones surface as a
# red docs-only CI run — fail-loud, never a silent skip — and must
# join this list.
# Modules that mention a cargo argv but only write FAKE cargo
# executables into scratch PATH fixtures (verified by reading them):
# they never run real cargo, so the docs-only lane may run them.
CARGO_FAKE_FIXTURE_MODULES = {
    "test_lint.py",
}

CARGO_DRIVER_MODULES = {
    "test_alphagenome_proto_generation.py",
    "test_build_identity_rebuild.py",
    "test_short_unix_socket_contract.py",
    "test_sdlc_clean_contract.py",
    "test_source_package_boundary.py",
    "test_ticket_401_surface_ratchets.py",
    # Toolchain drivers (bubblewrap, offline runner, lint bootstrap):
    "test_offline_gate_contract.py",
    "test_quality_ratchet_contract.py",
}
NEEDS_BINARY_MODULES_PATTERN = re.compile(
    r"BIOMCP_BIN|target/release|target/debug|target/spec"
)



def test_the_binary_dependent_modules_carry_the_marker() -> None:
    """The docs-only lane deselects these modules by marker; a new
    binary-dependent module must join the list (and this test) or its
    tests will fail on docs-only runners that have no binary.
    """
    for path in sorted((REPO_ROOT / "tests").rglob("test_*.py")):
        if path.name == "test_ci_workflow_contract.py":
            continue  # this scan's own literals must not match itself
        text = path.read_text(encoding="utf-8")
        import ast

        needs = bool(NEEDS_BINARY_MODULES_PATTERN.search(text))
        if not needs and re.search(r'[\"\']cargo[\"\']', text):
            needs = path.name not in CARGO_FAKE_FIXTURE_MODULES
        if path.name in CARGO_DRIVER_MODULES:
            needs = True
        if needs:
            # The marker must be a real module statement, not a
            # comment or string mentioning it.
            def _marker_in(value: object) -> bool:
                # Both spellings count: pytest.mark.needs_binary used
                # bare (an Attribute) and pytest.mark.needs_binary()
                # (a Call). A comment or string never parses as
                # either.
                if isinstance(value, ast.Attribute):
                    return getattr(value, "attr", "") == "needs_binary"
                if isinstance(value, ast.Call):
                    if getattr(value.func, "attr", "") == "needs_binary":
                        return True
                    return (
                        getattr(value.func, "attr", "") == "mark"
                        and any(
                            getattr(a, "attr", "") == "needs_binary"
                            for a in value.args
                        )
                    )
                if isinstance(value, (ast.List, ast.Tuple)):
                    return any(_marker_in(elt) for elt in value.elts)
                return False

            has_marker = any(
                _marker_in(node.value)
                for node in ast.walk(ast.parse(text))
                if isinstance(node, ast.Assign)
                and any(
                    getattr(t, "id", "") == "pytestmark"
                    for t in node.targets
                    if isinstance(t, ast.Name)
                )
            )
            assert has_marker, (
                f"{path} drives a built binary or cargo; it needs the "
                "needs_binary marker or the docs-only CI lane will fail it"
            )
