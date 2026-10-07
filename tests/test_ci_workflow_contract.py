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
DOC = yaml.safe_load(WORKFLOW.read_text(encoding="utf-8"))

RUST_JOBS = [
    "canonical-gates",
    "stress-lane",
    "release-panic",
    "full-features",
    "generated-sources",
    "windows-contracts",
]
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
    stripped = allow_list.replace("sdlc/*|notes/*|AGENTS.md|.github/*.md)", "")
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


def _assert_no_moving_ubuntu_label(text: str) -> None:
    assert "ubuntu-latest" not in text, (
        "ubuntu-latest is a moving label: GitHub moves it to Ubuntu 26 "
        "on 2026-10-19, which can change a release without a repository "
        "change. Pin the image version (ubuntu-24.04) instead."
    )


def test_no_workflow_job_runs_on_the_moving_ubuntu_latest_label() -> None:
    """Ticket 2024: every workflow job runs on a pinned image.

    The 2026-10-07 review found four release and contract jobs still
    on ubuntu-latest while every other job pinned ubuntu-24.04. The
    ban covers the file text, not only runs-on keys, so a matrix or
    a reuse block cannot smuggle the label back in either.
    """
    for path in sorted(WORKFLOWS.glob("*.yml")):
        _assert_no_moving_ubuntu_label(path.read_text(encoding="utf-8"))


def test_the_ubuntu_label_ban_catches_a_planted_ubuntu_latest() -> None:
    """Red proof: planting the moving label anywhere fails the ban."""
    with pytest.raises(AssertionError, match="moving label"):
        _assert_no_moving_ubuntu_label("    runs-on: ubuntu-latest\n")


def test_every_rust_job_waits_on_the_changes_job() -> None:
    for jid in RUST_JOBS:
        job = _job(jid)
        assert job.get("needs") == "changes", f"{jid} must need the changes job"
        assert "needs.changes.outputs.docs_only != 'true'" in str(job.get("if", "")), (
            f"{jid} must skip only docs-only pushes"
        )


def test_the_changes_job_covers_the_whole_push() -> None:
    changes = _job("changes")
    assert changes["outputs"]["docs_only"] == "${{ steps.classify.outputs.docs_only }}"
    run_step = next(s for s in changes["steps"] if s.get("id") == "classify")
    assert run_step["run"] == "scripts/ci-classify-push.sh"
    assert run_step["env"]["PUSH_BASE_REF"] == "origin/main"
    assert run_step["env"]["PUSH_AFTER"] == "${{ github.sha }}"


def test_the_cache_follows_checkout_and_toolchain_and_is_pinned() -> None:
    for jid in RUST_JOBS:
        checkout = _step_index(jid, "actions/checkout")
        toolchain = _step_index(jid, "rust-toolchain")
        cache = _step_index(jid, "Cache the Rust build")
        assert cache > checkout, f"{jid}: the cache must restore after checkout"
        assert cache > toolchain, (
            f"{jid}: the cache must run after the pinned toolchain so its key "
            "tracks the pinned rustc, not the runner default"
        )
        cache_step = _job(jid)["steps"][cache]
        assert cache_step["uses"] == CACHE_PIN, f"{jid}: the cache must be pinned"
        assert cache_step["with"]["cache-workspace-crates"] is True, (
            f"{jid}: the workspace crate must be cached or every run rebuilds it"
        )


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
    text = WORKFLOW.read_text(encoding="utf-8")
    assert "get.nexte.st" not in text, "the pipe-to-shell installer must be gone"
    assert text.count("scripts/install-nextest.sh") >= 3
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
    "test_pre_commit_reject_march_artifacts.py",  # names a scratch cargo.log; runs no cargo
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
