"""CI workflow contracts (2026-09-29 review).

The review of the cache round found the docs-only skip unsound
(decided by commit message, skippable by a mislabeled push, and
skipping the docs tests docs-only commits need) and the cache
placement wrong (before checkout and before the toolchain install,
unpinned, without workspace-crate caching). These tests pin the
corrected shapes so drift fails loudly.
"""

from __future__ import annotations

import ast
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
    # The base is the merge-base with origin/main (a failed tip
    # followed by a markdown commit must not skip the Rust jobs),
    # and executable markdown (spec/, skills/, src/, the compiled
    # CLI reference) never counts as docs.
    assert "git merge-base" in classify
    assert "sdlc/*|notes/*" in classify
    allow_list = classify.split("case")[1].split("esac")[0]
    for executable in ("docs/*", "README.md|", "|README"):
        assert executable not in allow_list.replace("sdlc/*|notes/*|CHANGELOG.md|AGENTS.md|.github/*.md)", ""), (
            "docs/ and README.md are read by Rust tests and compiled in; they must run full CI"
        )


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
    assert not any("check-test-wait-ratchet.py" in r for r in runs)
    assert "check-test-wait-ratchet.py" not in (REPO_ROOT / "Makefile").read_text()


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
    "test_prepare_biodata_release_dependency.py",  # executes scratch cargo and git stubs only
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
        if path.name == "test_biodata_branch_workflow.py":
            # This 1.0 runner contract uses a temporary stub and mocked cargo;
            # it requires no built BioMCP binary. Keep it in docs-only CI.
            continue
        needs = bool(NEEDS_BINARY_MODULES_PATTERN.search(text))
        if not needs and re.search(r'[\"\']cargo[\"\']', text):
            needs = path.name not in CARGO_FAKE_FIXTURE_MODULES
        if path.name in CARGO_DRIVER_MODULES:
            needs = True
        if needs:
            assert _has_module_binary_marker(text), (
                f"{path} drives a built binary or cargo; it needs the "
                "needs_binary marker or the docs-only CI lane will fail it"
            )


def _has_module_binary_marker(text: str) -> bool:
    """Accept actual pytest markers assigned at module scope."""
    def marker_in(value: ast.expr) -> bool:
        if isinstance(value, ast.Call):
            return marker_in(value.func)
        if isinstance(value, (ast.List, ast.Tuple)):
            return any(marker_in(elt) for elt in value.elts)
        return (
            isinstance(value, ast.Attribute)
            and value.attr == "needs_binary"
            and isinstance(value.value, ast.Attribute)
            and value.value.attr == "mark"
            and isinstance(value.value.value, ast.Name)
            and value.value.value.id == "pytest"
        )

    return any(
        marker_in(node.value)
        for node in ast.parse(text).body
        if isinstance(node, ast.Assign)
        and any(isinstance(t, ast.Name) and t.id == "pytestmark" for t in node.targets)
    )


def test_binary_marker_requires_a_real_module_assignment() -> None:
    for text in (
        "pytestmark = pytest.mark.needs_binary",
        "pytestmark = [pytest.mark.needs_binary]",
        "pytestmark = (pytest.mark.needs_binary(),)",
    ):
        assert _has_module_binary_marker(text), text
    for text in (
        "# pytestmark = pytest.mark.needs_binary",
        'pytestmark = "pytest.mark.needs_binary"',
        "pytestmark = other.needs_binary",
        "def test_local():\n    pytestmark = pytest.mark.needs_binary",
        "if False:\n    pytestmark = pytest.mark.needs_binary",
    ):
        assert not _has_module_binary_marker(text), text
