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

import yaml

REPO_ROOT = Path(__file__).resolve().parents[1]
WORKFLOW = REPO_ROOT / ".github" / "workflows" / "ci.yml"
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
    # The docs-only class is *.md, sdlc/ and notes/ — everything
    # else is a full-CI push.
    assert "docs_only=false" in classify
    assert 'case "$path" in' in classify


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
    assert run_step["env"]["PUSH_BEFORE"] == "${{ github.event.before }}"
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
        text = path.read_text(encoding="utf-8")
        if path.name == "test_biodata_branch_workflow.py":
            # This 1.0 runner contract uses a temporary stub and mocked cargo;
            # it requires no built BioMCP binary. Keep it in docs-only CI.
            continue
        needs = bool(NEEDS_BINARY_MODULES_PATTERN.search(text))
        if needs or path.name in CARGO_DRIVER_MODULES:
            needs = True
        if needs:
            assert "pytest.mark.needs_binary" in text, (
                f"{path} drives a built binary or cargo; it needs the "
                "needs_binary marker or the docs-only CI lane will fail it"
            )
