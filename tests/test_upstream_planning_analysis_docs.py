from __future__ import annotations

import os
import re
from pathlib import Path

import pytest

REPO_ROOT = Path(__file__).resolve().parents[1]
DEFAULT_PLANNING_ROOT = REPO_ROOT / "tests" / "fixtures" / "planning" / "biomcp"


def _read_repo(path: str) -> str:
    return (REPO_ROOT / path).read_text(encoding="utf-8")


def _planning_root() -> Path:
    return Path(os.environ.get("BIOMCP_PLANNING_ROOT", DEFAULT_PLANNING_ROOT))


def _read_planning(path: str) -> str:
    return (_planning_root() / path).read_text(encoding="utf-8")


def _workflow_job_block(workflow: str, job_name: str) -> str:
    match = re.search(
        rf"^  {re.escape(job_name)}:\n(.*?)(?=^  [A-Za-z0-9_-]+:\n|\Z)",
        workflow,
        flags=re.MULTILINE | re.DOTALL,
    )
    assert match is not None, f"missing workflow job {job_name}"
    return match.group(1)


def _workflow_run_steps(job_block: str) -> list[str]:
    return re.findall(r"^\s+- run: (.+)$", job_block, flags=re.MULTILINE)


def test_planning_contract_uses_repo_fixture_fallback_by_default(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    monkeypatch.delenv("BIOMCP_PLANNING_ROOT", raising=False)

    assert _planning_root() == DEFAULT_PLANNING_ROOT
    assert "# BioMCP Strategy" in _read_planning("strategy.md")


def test_planning_contract_reads_explicit_env_override(
    monkeypatch: pytest.MonkeyPatch, tmp_path: Path
) -> None:
    monkeypatch.setenv("BIOMCP_PLANNING_ROOT", str(tmp_path))
    (tmp_path / "strategy.md").write_text("# override strategy\n", encoding="utf-8")

    assert _planning_root() == tmp_path
    assert _read_planning("strategy.md") == "# override strategy\n"


def test_planning_contract_bad_override_fails_loudly(
    monkeypatch: pytest.MonkeyPatch, tmp_path: Path
) -> None:
    missing_root = tmp_path / "missing-planning-root"
    monkeypatch.setenv("BIOMCP_PLANNING_ROOT", str(missing_root))

    with pytest.raises(FileNotFoundError):
        _read_planning("strategy.md")


def test_strategy_and_frontier_capture_upstream_planning_contract() -> None:
    strategy = _read_planning("strategy.md")
    frontier = _read_planning("frontier.md")

    assert "# BioMCP Strategy" in strategy
    assert "Rust core, Python packaging" in strategy
    assert "Rate limiting is process-local" in strategy
    assert "G002" in strategy
    assert "G003" in strategy
    assert len(strategy.splitlines()) <= 80

    assert "# BioMCP Frontier" in frontier
    assert "## G002" in frontier
    assert "## G003" in frontier
    assert "design/functional/overview.md" in frontier
    assert "design/technical/overview.md" in frontier
    assert "design/ux/cli-reference.md" in frontier
    assert "Harvest Guidance" in frontier


def test_functional_overview_preserves_readme_surface_and_study_family() -> None:
    functional = _read_repo("design/functional/overview.md")

    assert "# BioMCP Functional Overview" in functional
    assert "## Entity Surface" in functional
    for entity in (
        "| gene |",
        "| variant |",
        "| article |",
        "| trial |",
        "| drug |",
        "| disease |",
        "| pathway |",
        "| protein |",
        "| adverse-event |",
        "| pgx |",
        "| gwas |",
        "| phenotype |",
    ):
        assert entity in functional

    assert "## Study Command Family" in functional
    assert "`study` is a separate local analytics surface" in functional
    assert (
        "`biomcp study list|download|filter|query|co-occurrence|cohort|survival|compare`"
        in functional
    )
    assert "BioMCP ships an embedded agent guide" in functional
    assert "`biomcp skill` shows the BioMCP agent guide" in functional
    assert "`biomcp skill install <dir>` installs that guide" in functional
    assert "`biomcp skill list` is a legacy compatibility alias" in functional
    assert "`No skills found`" in functional
    assert "`biomcp skill 03` fail clearly" in functional
    assert "search all [slot filters]" in functional
    assert "biomcp search all --gene BRAF --disease melanoma" in functional
    assert "biomcp search all BRAF" in functional


def test_technical_and_ux_docs_match_current_cli_and_workflow_contracts() -> None:
    technical = _read_repo("design/technical/overview.md")
    ux = _read_repo("design/ux/cli-reference.md")

    assert "CI (`.github/workflows/ci.yml`) runs five parallel jobs" in technical
    assert "`check` (`cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test`)" in technical
    assert "`version-sync` (`bash scripts/check-version-sync.sh`)" in technical
    assert "`climb-hygiene` (`bash scripts/check-no-climb-tracked.sh`)" in technical
    assert (
        '`contracts` (`cargo build --release --locked`, `uv sync --extra dev`, '
        '`uv run pytest tests/ -v --mcp-cmd "./target/release/biomcp serve"`, '
        '`uv run mkdocs build --strict`)'
        in technical
    )
    assert "`spec-stable` (`cargo build --release --locked`, then `make spec-pr`)" in technical
    assert "PR CI now runs `make spec-pr` via the `spec-stable` job in `.github/workflows/ci.yml`." in technical
    assert "Volatile live-network headings run separately in `.github/workflows/spec-smoke.yml`" in technical
    assert "Contract smoke checks run in `.github/workflows/contracts.yml`" in technical
    assert "release validation runs `pytest tests/` and `mkdocs build --strict`" in technical
    assert "Streamable HTTP" in technical
    assert "`/mcp`" in technical
    assert "`/health`" in technical
    assert "`/readyz`" in technical
    assert "connection line and `Command:` markers" in technical
    assert "through the remote" in technical
    assert "`biomcp` tool" in technical
    assert "remote `shell`" not in technical

    assert "`search all` Contract" in ux
    assert "typed slots first" in ux
    assert "biomcp search all --gene BRAF --disease melanoma" in ux
    assert 'biomcp search all --keyword "checkpoint inhibitor"' in ux
    assert "biomcp search all BRAF" in ux
    assert "positional alias" in ux
    assert "biomcp skill                  → show the embedded BioMCP agent guide" in ux
    assert "biomcp skill list             → legacy alias; currently reports no embedded catalog" in ux
    assert "Overview: `biomcp skill`" in ux
    assert "List: `biomcp skill list`" in ux
    assert "Legacy lookup: `biomcp skill 03` or `biomcp skill variant-to-treatment`" in ux
    assert "biomcp serve-http            → run the MCP Streamable HTTP server at `/mcp`" in ux
    assert "biomcp serve-sse             → removed compatibility command; use `biomcp serve-http`" in ux


def test_source_integration_architecture_doc_captures_repo_contract() -> None:
    technical = _read_repo("design/technical/overview.md")
    source_integration = _read_repo("design/technical/source-integration.md")

    assert "source-integration.md" in technical
    assert "# BioMCP Source Integration Architecture" in source_integration
    assert "## New Source vs Existing Source" in source_integration
    assert "`src/sources/<source>.rs`" in source_integration
    assert "`src/sources/mod.rs`" in source_integration
    assert "`shared_client()`" in source_integration
    assert "`streaming_http_client()`" in source_integration
    assert "`env_base(default, ENV_VAR)`" in source_integration
    assert "`read_limited_body()`" in source_integration
    assert "`body_excerpt()`" in source_integration
    assert "`retry_send()`" in source_integration
    assert "## Section-First Entity Integration" in source_integration
    assert "`src/cli/mod.rs`" in source_integration
    assert "`src/cli/list.rs`" in source_integration
    assert "`docs/user-guide/cli-reference.md`" in source_integration
    assert "default `get` output stays concise" in source_integration
    assert "## Provenance and Rendering" in source_integration
    assert "`source_label`" in source_integration
    assert "source-specific notes" in source_integration
    assert "## Auth, Cache, and Secrets" in source_integration
    assert "`BioMcpError::ApiKeyRequired`" in source_integration
    assert "`apply_cache_mode_with_auth(..., true)`" in source_integration
    assert "`docs/getting-started/api-keys.md`" in source_integration
    assert "`docs/reference/data-sources.md`" in source_integration
    assert "Do not log secrets" in source_integration
    assert "## Graceful Degradation and Timeouts" in source_integration
    assert "Optional enrichments must not take down the whole command" in source_integration
    assert "truthful about missing or unavailable data" in source_integration
    assert "## Rate Limits and Operational Constraints" in source_integration
    assert "`biomcp serve-http`" in source_integration
    assert "process-local" in source_integration
    assert "## Source Addition Checklist" in source_integration
    assert "`docs/reference/source-versioning.md`" in source_integration
    assert "`src/cli/health.rs`" in source_integration
    assert "`scripts/contract-smoke.sh`" in source_integration
    assert "`spec/`" in source_integration
    assert "`CHANGELOG.md`" in source_integration


def test_pull_request_contract_gate_matches_release_validation() -> None:
    ci = _read_repo(".github/workflows/ci.yml")
    release = _read_repo(".github/workflows/release.yml")
    contracts_smoke = _read_repo(".github/workflows/contracts.yml")
    spec_smoke = _read_repo(".github/workflows/spec-smoke.yml")
    expected_ci_contract_runs = [
        "cargo build --release --locked",
        "uv sync --extra dev",
        'uv run pytest tests/ -v --mcp-cmd "./target/release/biomcp serve"',
        "uv run mkdocs build --strict",
    ]
    expected_release_contract_runs = [
        "uv sync --extra dev",
        'uv run pytest tests/ -v --mcp-cmd "biomcp serve"',
        "uv run mkdocs build --strict",
    ]

    ci_contracts = _workflow_job_block(ci, "contracts")
    ci_spec = _workflow_job_block(ci, "spec-stable")
    ci_version_sync = _workflow_job_block(ci, "version-sync")
    ci_climb_hygiene = _workflow_job_block(ci, "climb-hygiene")
    release_validate = _workflow_job_block(release, "validate")
    smoke_spec = _workflow_job_block(spec_smoke, "spec-volatile-live")

    assert 'python-version: "3.12"' in ci_contracts
    assert 'python-version: "3.12"' in ci_spec
    assert 'python-version: "3.12"' in release_validate
    assert 'python-version: "3.12"' in smoke_spec
    assert _workflow_run_steps(ci_contracts) == expected_ci_contract_runs
    assert "- uses: actions/checkout@v4" in ci_spec
    assert "uses: arduino/setup-protoc@v3" in ci_spec
    assert "uses: dtolnay/rust-toolchain@stable" in ci_spec
    assert "uses: actions/setup-python@v5" in ci_spec
    assert "uses: astral-sh/setup-uv@v4" in ci_spec
    assert _workflow_run_steps(ci_spec) == [
        "cargo build --release --locked",
        "make spec-pr",
    ]
    assert _workflow_run_steps(release_validate)[-3:] == expected_release_contract_runs
    assert "- uses: actions/checkout@v4" in ci_version_sync
    assert _workflow_run_steps(ci_version_sync) == [
        "bash scripts/check-version-sync.sh"
    ]
    for forbidden in (
        "setup-python",
        "setup-uv",
        "setup-protoc",
        "rust-toolchain",
        "cargo ",
        "uv sync",
        "python-version:",
    ):
        assert forbidden not in ci_version_sync
    assert "- uses: actions/checkout@v4" in ci_climb_hygiene
    assert _workflow_run_steps(ci_climb_hygiene) == [
        "bash scripts/check-no-climb-tracked.sh"
    ]
    for forbidden in (
        "setup-python",
        "setup-uv",
        "setup-protoc",
        "rust-toolchain",
        "cargo ",
        "uv sync",
        "python-version:",
    ):
        assert forbidden not in ci_climb_hygiene

    assert "name: Contract Smoke Tests" in contracts_smoke
    assert 'cron: "0 6 * * *"' in contracts_smoke
    assert "workflow_dispatch:" in contracts_smoke
    assert "continue-on-error: true" in contracts_smoke
    assert "- run: bash scripts/contract-smoke.sh" in contracts_smoke

    assert "name: Spec smoke (volatile live-network)" in spec_smoke
    assert 'cron: "0 7 * * *"' in spec_smoke
    assert "workflow_dispatch:" in spec_smoke
    assert "- uses: actions/checkout@v4" in smoke_spec
    assert "uses: arduino/setup-protoc@v3" in smoke_spec
    assert "uses: dtolnay/rust-toolchain@stable" in smoke_spec
    assert "uses: actions/setup-python@v5" in smoke_spec
    assert "uses: astral-sh/setup-uv@v4" in smoke_spec
    assert _workflow_run_steps(smoke_spec) == [
        "cargo build --release --locked",
        "make spec",
    ]


def test_makefile_spec_split_contract_is_documented_and_executable() -> None:
    makefile = _read_repo("Makefile")

    assert ".PHONY: build test lint check run clean spec spec-pr validate-skills test-contracts" in makefile
    assert "Volatile live-network spec headings." in makefile
    assert "PR gate: repo-local checks plus live-backed headings that have been stable" in makefile
    assert "Smoke lane: `search article`, `gene articles`, `variant articles`," in makefile
    assert "To move a heading into the smoke lane, add its exact pytest markdown node ID" in makefile
    assert 'SPEC_PR_DESELECT_ARGS = \\' in makefile
    for node_id in (
        'spec/02-gene.md::Gene to Articles',
        'spec/03-variant.md::Variant to Articles',
        'spec/06-article.md::Searching by Gene',
        'spec/06-article.md::Searching by Keyword',
        'spec/06-article.md::Sort Behavior',
        'spec/07-disease.md::Disease to Articles',
    ):
        assert f'--deselect "{node_id}"' in makefile
    assert re.search(
        r"^spec-pr:\n\tXDG_CACHE_HOME=\"\$\(CURDIR\)/\.cache\" PATH=\"\$\(CURDIR\)/target/release:\$\(PATH\)\" \\\n\t\tuv run --extra dev sh -c 'PATH=\"\$\(CURDIR\)/target/release:\$\$PATH\" pytest spec/ --mustmatch-lang bash --mustmatch-timeout 60 -v \$\(SPEC_PR_DESELECT_ARGS\)'$",
        makefile,
        flags=re.MULTILINE,
    )
    assert re.search(
        r"^test-contracts:\n"
        r"\tcargo build --release --locked\n"
        r"\tuv sync --extra dev\n"
        r'\tuv run pytest tests/ -v --mcp-cmd "\./target/release/biomcp serve"\n'
        r"\tuv run mkdocs build --strict$",
        makefile,
        flags=re.MULTILINE,
    )


def test_runtime_contract_docs_and_scripts_align_on_release_target() -> None:
    staging_demo = _read_repo("design/technical/staging-demo.md")
    runbook = _read_repo("RUN.md")
    technical = _read_repo("design/technical/overview.md")
    scripts_readme = _read_repo("scripts/README.md")
    source_contracts = _read_repo("scripts/source-contracts.md")
    contract_smoke = _read_repo("scripts/contract-smoke.sh")
    genegpt_demo = _read_repo("scripts/genegpt-demo.sh")
    geneagent_demo = _read_repo("scripts/geneagent-demo.sh")

    assert "# BioMCP Staging and Demo Contract" in staging_demo
    assert "./target/release/biomcp" in staging_demo
    assert "shared merged-main target" in staging_demo
    assert "BIOMCP_BIN=./target/release/biomcp ./scripts/genegpt-demo.sh" in staging_demo
    assert "BIOMCP_BIN=./target/release/biomcp ./scripts/geneagent-demo.sh" in staging_demo
    assert "./scripts/contract-smoke.sh --fast" in staging_demo
    assert 'uv run pytest tests/test_mcp_contract.py -v --mcp-cmd "./target/release/biomcp serve"' in staging_demo
    assert "ONCOKB_TOKEN" in staging_demo
    assert "./target/release/biomcp serve-http --host 127.0.0.1 --port 8080" in staging_demo
    assert "POST/GET /mcp" in staging_demo
    assert "GET /health" in staging_demo
    assert "GET /readyz" in staging_demo
    assert "GET /" in staging_demo
    assert "tests/test_mcp_http_transport.py" in staging_demo
    assert "S2_API_KEY" in staging_demo
    assert "article citations 22663011 --limit 3" in staging_demo

    assert "# BioMCP Runbook" in runbook
    assert "cargo build --release --locked" in runbook
    assert "./target/release/biomcp serve" in runbook
    assert "./target/release/biomcp serve-http --host 127.0.0.1 --port 8080" in runbook
    assert 'uv run pytest tests/test_mcp_contract.py -v --mcp-cmd "./target/release/biomcp serve"' in runbook
    assert "curl http://127.0.0.1:8080/health" in runbook
    assert "curl http://127.0.0.1:8080/readyz" in runbook
    assert "curl http://127.0.0.1:8080/" in runbook
    assert "tests/test_mcp_http_surface.py" in runbook
    assert "tests/test_mcp_http_transport.py" in runbook
    assert "make spec" in runbook
    assert "make test-contracts" in runbook
    assert "S2_API_KEY" in runbook
    assert "./target/release/biomcp article citations 22663011 --limit 3" in runbook
    assert (
        '`make test-contracts` runs `cargo build --release --locked`, '
        '`uv sync --extra dev`, `pytest tests/ -v --mcp-cmd "./target/release/biomcp serve"`, '
        'and `mkdocs build --strict` - the same steps that PR CI `contracts` requires.'
        in runbook
    )
    assert "docs/user-guide/cli-reference.md" in runbook
    assert "docs/reference/mcp-server.md" in runbook

    assert "design/technical/staging-demo.md" in technical
    assert "RUN.md" in technical
    assert "S2_API_KEY" in technical
    assert "Semantic Scholar article enrichment/navigation" in technical
    assert "No `RUN.md` or staging-demo runbook exists" not in technical

    assert "current BioMCP operator command layer" in scripts_readme
    assert "source-facing contract probes" in scripts_readme
    assert "091 expansion scope" not in scripts_readme

    assert "# BioMCP Source Contract Probes" in source_contracts
    assert "source-facing API contract probes" in source_contracts
    assert "ONCOKB_TOKEN" in source_contracts
    assert "Semantic Scholar" in source_contracts
    assert "S2_API_KEY" in source_contracts
    assert "091 expansion scope" not in source_contracts

    assert "ONCOKB_TOKEN" in contract_smoke
    assert "ONCOKB_API_TOKEN" in contract_smoke
    assert "S2_API_KEY" in contract_smoke
    assert "Semantic Scholar" in contract_smoke
    assert "set ONCOKB_TOKEN to enable" in contract_smoke

    for demo_script in (genegpt_demo, geneagent_demo):
        assert 'BIN="${BIOMCP_BIN:-' in demo_script
        assert "$ROOT/target/release/biomcp" in demo_script
        assert "target/debug/biomcp" not in demo_script
        assert 'command -v biomcp >/dev/null 2>&1' in demo_script
