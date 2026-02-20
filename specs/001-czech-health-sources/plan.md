# Implementation Plan: CzechMedMCP - Czech Healthcare Data Sources

**Branch**: `001-czech-health-sources` | **Date**: 2026-02-17 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/001-czech-health-sources/spec.md`

## Summary

Fork BioMCP (MIT) and extend it with 14 MVP Czech healthcare MCP tools
across 5 modules (SUKL, MKN-10, NRPZS, SZV, VZP). The Czech modules
are isolated under `src/biomcp/czech/` with minimal changes to the
existing codebase (5-10 lines across 5 files). All existing 21 BioMCP
tools remain fully functional. The approach is additive, not
destructive, enabling upstream sync via `git merge upstream/main`.

## Technical Context

**Language/Version**: Python >= 3.10, < 4.0
**Primary Dependencies**: mcp[cli] >= 1.12.3, httpx, pydantic v2, lxml (new), diskcache
**Storage**: diskcache for API response caching, in-memory index for MKN-10 ClaML
**Testing**: pytest, pytest-asyncio, pytest-xdist, pytest-cov
**Target Platform**: Linux/macOS server, Docker, Cloudflare Workers
**Project Type**: Single project (extending existing BioMCP structure)
**Performance Goals**: Search < 2s (cold), < 100ms (cached), ClaML parse < 5s
**Constraints**: No authentication required for public APIs, MIT license
**Scale/Scope**: 4,000+ physicians via Medevio, 35 total MCP tools (14 Czech MVP + 21 global)

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Evidence |
|-----------|--------|----------|
| **I. MCP Protocol First** | PASS | All 14 Czech tools registered as MCP tools via `@mcp_app.tool()` |
| **II. Modular Domain Architecture** | PASS | Czech modules isolated in `src/biomcp/czech/` with no sibling imports. Shared infra via top-level utilities. Each module follows `__init__.py` + `search.py` + `getter.py` pattern |
| **III. Authoritative Data Sources** | PASS | SUKL, UZIS/MKN-10, NRPZS, MZCR/SZV, VZP are authoritative Czech government data sources. All endpoints documented in research.md. Data attributed to source |
| **IV. CLI & MCP Dual Access** | PASS | All Czech tools accessible via `biomcp czech` CLI commands. JSON + human-readable output supported |
| **V. Testing Rigor** | PASS | Unit tests with mocked HTTP. Integration tests marked `@pytest.mark.integration`. Test structure mirrors source under `tests/czech/`. Target >= 80% coverage |

**Constitution Check: PASS (all 5 principles satisfied)**

## Project Structure

### Documentation (this feature)

```text
specs/001-czech-health-sources/
├── plan.md              # This file
├── spec.md              # Feature specification
├── research.md          # Phase 0: API research and decisions
├── data-model.md        # Phase 1: Entity definitions
├── quickstart.md        # Phase 1: Setup and usage guide
├── contracts/
│   └── mcp-tools.md     # Phase 1: MCP tool contracts
└── checklists/
    └── requirements.md  # Spec quality validation
```

### Source Code (repository root)

```text
src/biomcp/
├── czech/                      # NEW: All Czech modules
│   ├── __init__.py             # Czech sub-package init
│   ├── czech_tools.py          # MCP tool registrations (14 tools)
│   ├── diacritics.py           # Shared diacritics normalization
│   ├── sukl/
│   │   ├── __init__.py
│   │   ├── models.py           # Pydantic v2 models for Drug, DrugSearchResult
│   │   ├── search.py           # _sukl_drug_search() async function
│   │   ├── getter.py           # _sukl_drug_details(), _sukl_spc(), _sukl_pil()
│   │   └── availability.py     # _sukl_availability_check()
│   ├── mkn/
│   │   ├── __init__.py
│   │   ├── models.py           # Pydantic v2 models for Diagnosis, DiagnosisCategory
│   │   ├── parser.py           # ClaML XML parser + in-memory index
│   │   └── search.py           # _mkn_search(), _mkn_get(), _mkn_browse()
│   ├── nrpzs/
│   │   ├── __init__.py
│   │   ├── models.py           # Pydantic v2 models for HealthcareProvider
│   │   └── search.py           # _nrpzs_search(), _nrpzs_get()
│   ├── szv/
│   │   ├── __init__.py
│   │   ├── models.py           # Pydantic v2 models for HealthProcedure
│   │   └── search.py           # _szv_search(), _szv_get()
│   └── vzp/
│       ├── __init__.py
│       ├── models.py           # Pydantic v2 models for CodebookEntry
│       └── search.py           # _vzp_search(), _vzp_get()
├── cli/
│   └── czech.py                # NEW: CLI commands for Czech tools
├── core.py                     # MODIFIED: Rename to "CzechMedMCP" (1 line)
├── __init__.py                 # MODIFIED: Add czech import (1 line)
├── constants.py                # MODIFIED: Add 5 Czech domains (5 lines)
└── domain_handlers.py          # MODIFIED: Add 5 Czech handlers (~50 lines)

tests/
├── czech/                      # NEW: All Czech tests
│   ├── __init__.py
│   ├── conftest.py             # Shared fixtures, mock data
│   ├── test_sukl_search.py
│   ├── test_sukl_getter.py
│   ├── test_sukl_availability.py
│   ├── test_mkn_parser.py
│   ├── test_mkn_search.py
│   ├── test_nrpzs_search.py
│   ├── test_nrpzs_getter.py
│   ├── test_szv_search.py
│   ├── test_szv_getter.py
│   ├── test_vzp_search.py
│   ├── test_vzp_getter.py
│   └── test_diacritics.py
└── czech_integration/          # NEW: Integration tests
    ├── __init__.py
    ├── test_sukl_api.py        # @pytest.mark.integration
    ├── test_nrpzs_api.py       # @pytest.mark.integration
    └── test_szv_api.py         # @pytest.mark.integration
```

**Structure Decision**: Extends existing BioMCP single-project structure.
Czech code is entirely additive under `src/biomcp/czech/` and `tests/czech/`.
The 5 modified existing files (core.py, __init__.py, constants.py, domain_handlers.py, pyproject.toml) change a total of ~60 lines.

## Complexity Tracking

> No Constitution Check violations. No complexity justification needed.

---

## Phase Dependencies

```
Phase 1: Foundation (rebrand, structure, deps)
    │
    ├──> Phase 2: SUKL Module (P1 - highest value)
    │        │
    │        ├──> Phase 3: MKN-10 Module (P2 - independent)
    │        │
    │        ├──> Phase 4: NRPZS Module (P3 - independent)
    │        │
    │        └──> Phase 5: SZV + VZP Modules (P4 - independent)
    │
    └──> Phase 6: Integration, CLI, Tests, Release
```

Phases 2-5 can run in parallel after Phase 1. Phase 6 depends on all
previous phases completing.
