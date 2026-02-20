<!--
Sync Impact Report
===================
Version change: N/A (template) -> 1.0.0
Modified principles: N/A (initial creation)
Added sections:
  - Core Principles (5): MCP Protocol First, Modular Domain Architecture,
    Authoritative Data Sources, CLI & MCP Dual Access, Testing Rigor
  - Technical Constraints
  - Development Workflow
  - Governance
Removed sections: N/A
Templates requiring updates:
  - .specify/templates/plan-template.md: ✅ compatible (Constitution Check section exists)
  - .specify/templates/spec-template.md: ✅ compatible (no changes needed)
  - .specify/templates/tasks-template.md: ✅ compatible (no changes needed)
Follow-up TODOs: None
-->

# BioMCP Constitution

## Core Principles

### I. MCP Protocol First

Every feature MUST be implemented as an MCP-compliant tool or resource.
The Model Context Protocol specification is the authoritative interface
contract. All tools MUST follow MCP input/output conventions and MUST be
registered through the MCP server. New functionality that cannot be
expressed as an MCP tool MUST be justified in writing before implementation.

**Rationale**: BioMCP exists to bridge AI assistants and biomedical data.
MCP compliance ensures interoperability with any MCP-compatible client
(Claude Desktop, agents, third-party integrations).

### II. Modular Domain Architecture

Each biomedical domain (articles, trials, variants, genes, diseases,
drugs, biomarkers, organizations, interventions, openfda) MUST be
implemented as an independent module under `src/biomcp/`. Modules MUST
NOT import from sibling domain modules directly. Shared functionality
MUST live in top-level utility modules (`http_client.py`,
`cbioportal_helper.py`, `exceptions.py`, etc.). New domains MUST follow
the existing pattern: `__init__.py` + domain-specific files (search,
getter, fetch).

**Rationale**: Independent modules enable parallel development, isolated
testing, and clear ownership. Adding a new data source MUST NOT require
modifying existing domain modules.

### III. Authoritative Data Sources

BioMCP MUST integrate only with established, authoritative biomedical
databases and APIs (PubMed, ClinicalTrials.gov, NCI, MyVariant.info,
MyGene.info, MyDisease.info, MyChem.info, cBioPortal, OncoKB, OpenFDA,
TCGA/GDC, Ensembl). Every external API endpoint MUST be documented in
`THIRD_PARTY_ENDPOINTS.md`. Data returned to the user MUST be attributed
to its source. BioMCP MUST NOT fabricate, interpolate, or infer
biomedical data that is not present in the source response.

**Rationale**: Biomedical data accuracy is safety-critical. Incorrect
variant annotations or trial eligibility data can have real-world
clinical consequences.

### IV. CLI & MCP Dual Access

Every search and fetch operation MUST be accessible via both the MCP
server (tools) and the CLI (`biomcp` command). The CLI module under
`src/biomcp/cli/` MUST mirror the MCP tool surface. Output MUST support
both JSON (machine-readable) and human-readable formats. CLI commands
MUST use stdout for data and stderr for errors/diagnostics.

**Rationale**: CLI access enables testing, debugging, scripting, and
use outside MCP contexts. Dual access ensures feature parity and
provides a validation layer for MCP tool behavior.

### V. Testing Rigor

All new features MUST include unit tests. Integration tests (tests that
call external APIs) MUST be marked with `@pytest.mark.integration` and
MUST be runnable independently of unit tests. Tests MUST NOT depend on
network availability for the unit test suite. Mocking external HTTP calls
is REQUIRED for unit tests. The test directory structure MUST mirror the
source structure under `tests/`. CI MUST pass all unit tests; integration
test failures MUST NOT block the build.

**Rationale**: BioMCP depends on numerous external APIs that are outside
our control. Separating unit and integration tests ensures fast,
reliable CI while still validating real API behavior in dedicated runs.

## Technical Constraints

- **Python**: 3.10+ (no lower bound negotiable)
- **Package manager**: `uv` (recommended), `pip` (supported)
- **HTTP client**: `httpx` with async support via centralized
  `http_client.py`
- **Data validation**: Pydantic v2 models for all API responses
- **MCP SDK**: `mcp[cli] >=1.12.3, <2.0.0`
- **Linting/Formatting**: `ruff` for linting and formatting
- **Type checking**: `mypy` for static type analysis
- **License**: MIT - all contributions MUST be MIT-compatible
- **Transport modes**: STDIO (default), SSE (legacy), Streamable HTTP
  (recommended for deployment)
- **External API tokens**: MUST be optional; core functionality MUST
  work without authentication (higher rate limits MAY require tokens)

## Development Workflow

- **Branching**: Feature branches from `main`, merged via pull request
- **Code review**: All PRs MUST be reviewed before merge
- **Testing gate**: `make test` (unit tests) MUST pass before merge;
  integration tests run separately and are advisory
- **Commit style**: Conventional commits preferred
  (`feat:`, `fix:`, `docs:`, `refactor:`, `test:`, `chore:`)
- **Documentation**: New MCP tools MUST be documented in `README.md`
  and relevant developer guides under `docs/`
- **Dependency management**: `pyproject.toml` is the single source of
  truth; use `uv` for dependency resolution
- **Pre-commit hooks**: `pre-commit` MUST be installed for local
  development (`ruff` checks enforced)

## Governance

This constitution is the authoritative reference for BioMCP development
decisions. It supersedes ad-hoc conventions and informal agreements.

**Amendment process**:
1. Propose the change in a PR modifying this file
2. Provide rationale and impact assessment
3. Obtain maintainer approval
4. Update version following semantic versioning (below)

**Versioning policy**:
- MAJOR: Principle removal or redefinition that changes project direction
- MINOR: New principle added or existing principle materially expanded
- PATCH: Clarifications, wording improvements, non-semantic changes

**Compliance review**:
- All PRs SHOULD be checked against applicable principles
- Violations MUST be documented and justified in PR description
- Complexity beyond the minimum required MUST be justified

**Version**: 1.0.0 | **Ratified**: 2026-02-17 | **Last Amended**: 2026-02-17
