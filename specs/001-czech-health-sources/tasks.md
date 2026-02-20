# Tasks: CzechMedMCP - Czech Healthcare Data Sources

**Input**: Design documents from `/specs/001-czech-health-sources/`
**Prerequisites**: plan.md (required), spec.md (required), research.md, data-model.md, contracts/mcp-tools.md, quickstart.md

**Tests**: Included - spec requires >= 80% test coverage (SC-005) and "Unit and integration tests for Czech modules" in scope.

**Organization**: Tasks grouped by user story. Each story maps to a Czech healthcare module.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Create Czech package structure, add dependencies, shared utilities

- [x] T001 Create Czech package directory structure: `src/biomcp/czech/__init__.py`, `src/biomcp/czech/sukl/__init__.py`, `src/biomcp/czech/mkn/__init__.py`, `src/biomcp/czech/nrpzs/__init__.py`, `src/biomcp/czech/szv/__init__.py`, `src/biomcp/czech/vzp/__init__.py`
- [x] T002 Create test directory structure: `tests/czech/__init__.py`, `tests/czech/conftest.py`, `tests/czech_integration/__init__.py`
- [x] T003 [P] Add `lxml` dependency to `pyproject.toml` in `[project] dependencies` section
- [x] T004 [P] Implement diacritics normalization utility in `src/biomcp/czech/diacritics.py` using `unicodedata.normalize('NFD', text)` for transparent Czech/ASCII search matching
- [x] T005 [P] Write unit tests for diacritics utility in `tests/czech/test_diacritics.py` - test "léky"="leky", "Ústí"="Usti", preservation of original text

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Minimal changes to existing BioMCP files to integrate Czech module. MUST complete before user stories.

**CRITICAL**: No user story work can begin until this phase is complete

- [x] T006 Add `from . import czech` import to `src/biomcp/__init__.py` and add `"czech"` to `__all__` list
- [x] T006b [P] Rename FastMCP app to "CzechMedMCP" in `src/biomcp/core.py` (1 line change in `mcp_app = FastMCP(...)`)
- [x] T006c [P] ~~Add 5 Czech domain handler classes to `src/biomcp/domain_handlers.py`~~ DEFERRED to v1.1 - Czech tools work as standalone MCP tools, unified search integration not needed for MVP
- [x] T007 [P] Add Czech API base URLs to `src/biomcp/constants.py`: SUKL_BASE_URL (`https://prehledy.sukl.cz`), NRPZS_BASE_URL (`https://nrpzs.uzis.cz/api/v1`), SZV_BASE_URL (`https://szv.mzcr.cz`), NZIP_BASE_URL (`https://nzip.cz`), VZP_BASE_URL (`https://www.vzp.cz`)
- [x] T008 [P] Create empty MCP tool registration module `src/biomcp/czech/czech_tools.py` with imports from `biomcp.core` (`mcp_app`) and `biomcp.metrics` (`track_performance`) - tools added per user story
- [x] T009 [P] Create Czech CLI module `src/biomcp/cli/czech.py` with `czech_app = typer.Typer()` and sub-apps for sukl, mkn, nrpzs, szv, vzp
- [x] T010 Register Czech CLI in `src/biomcp/cli/main.py`: import `czech_app` from `.czech` and add `app.add_typer(czech_app, name="czech", no_args_is_help=True)`
- [x] T011 Create shared test fixtures in `tests/czech/conftest.py`: mock httpx client fixture, sample SUKL API responses, sample NRPZS responses, sample ClaML XML snippet

**Checkpoint**: Foundation ready - user story implementation can now begin in parallel

---

## Phase 3: User Story 1 - Drug Information Lookup (Priority: P1) MVP

**Goal**: Physicians can search Czech drugs, get details, read SmPC/PIL, check availability via SUKL API

**Independent Test**: Search "Ibuprofen" via MCP tool, verify results match SUKL data. Get drug by SUKL code, retrieve SmPC/PIL text, check availability status.

### Tests for User Story 1

> **NOTE: Write these tests FIRST, ensure they FAIL before implementation**

- [x] T012 [P] [US1] Write unit tests for SUKL drug search in `tests/czech/test_sukl_search.py` - test query by name, ATC code, pagination, empty results, diacritics handling
- [x] T013 [P] [US1] Write unit tests for SUKL drug getter in `tests/czech/test_sukl_getter.py` - test drug details retrieval, SmPC text, PIL text, invalid SUKL code error
- [x] T014 [P] [US1] Write unit tests for SUKL availability in `tests/czech/test_sukl_availability.py` - test available/limited/unavailable statuses, invalid code error

### Implementation for User Story 1

- [x] T015 [P] [US1] Create Pydantic v2 models in `src/biomcp/czech/sukl/models.py`: `ActiveSubstance`, `AvailabilityStatus`, `Drug`, `DrugSummary`, `DrugSearchResult` per data-model.md
- [x] T016 [US1] Implement `_sukl_drug_search()` async function in `src/biomcp/czech/sukl/search.py` using SUKL DLP API v1, with diacritics normalization on query, diskcache for response caching and offline fallback
- [x] T017 [US1] Implement `_sukl_drug_details()`, `_sukl_spc_getter()`, `_sukl_pil_getter()` in `src/biomcp/czech/sukl/getter.py` - fetch full drug record, SmPC text, PIL text by SUKL code
- [x] T018 [US1] Implement `_sukl_availability_check()` in `src/biomcp/czech/sukl/availability.py` - check distribution status for a drug by SUKL code
- [x] T019 [US1] Register 5 SUKL MCP tools in `src/biomcp/czech/czech_tools.py`: `sukl_drug_searcher`, `sukl_drug_getter`, `sukl_spc_getter`, `sukl_pil_getter`, `sukl_availability_checker` using `@mcp_app.tool()` and `@track_performance()` decorators with `Annotated[type, Field()]` parameters per contracts/mcp-tools.md
- [x] T020 [US1] Add SUKL CLI commands in `src/biomcp/cli/czech.py`: `sukl search --query`, `sukl get <code>`, `sukl spc <code>`, `sukl pil <code>`, `sukl availability <code>` with `--format json|human` output (Constitution IV)
- [x] T021 [US1] Write integration test in `tests/czech_integration/test_sukl_api.py` marked `@pytest.mark.integration` - real HTTP calls to SUKL API for search and detail retrieval

**Checkpoint**: User Story 1 fully functional - drug search, details, SmPC, PIL, availability all working via MCP and CLI

---

## Phase 4: User Story 2 - Diagnosis Code Lookup (Priority: P2)

**Goal**: Physicians can search MKN-10 codes by code or free text, view diagnosis hierarchy, browse categories

**Independent Test**: Search "J06.9" returns correct diagnosis, free-text "angina" returns relevant codes, browse chapters returns hierarchy

### Tests for User Story 2

> **NOTE: Write these tests FIRST, ensure they FAIL before implementation**

- [x] T022 [P] [US2] Write unit tests for ClaML parser in `tests/czech/test_mkn_parser.py` - test XML parsing of Class, SuperClass, SubClass elements, hierarchy building, index creation from sample ClaML snippet
- [x] T023 [P] [US2] Write unit tests for MKN-10 search in `tests/czech/test_mkn_search.py` - test code search ("J06.9"), free-text search ("infarkt"), browse chapters, invalid code error, diacritics handling

### Implementation for User Story 2

- [x] T024 [P] [US2] Create Pydantic v2 models in `src/biomcp/czech/mkn/models.py`: `DiagnosisHierarchy`, `Modifier`, `Diagnosis`, `DiagnosisCategory` per data-model.md
- [x] T025 [US2] Implement ClaML XML parser in `src/biomcp/czech/mkn/parser.py` using `lxml.etree` - parse `<Class>`, `<SuperClass>`, `<SubClass>`, `<ModifierClass>` elements, build in-memory index with code-to-diagnosis mapping and text search index, cache parsed result with diskcache
- [x] T026 [US2] Implement `_mkn_search()`, `_mkn_get()`, `_mkn_browse()` in `src/biomcp/czech/mkn/search.py` - search by code/text using in-memory index, get full diagnosis with hierarchy, browse category tree
- [x] T027 [US2] Register 3 MKN-10 MCP tools in `src/biomcp/czech/czech_tools.py`: `mkn_diagnosis_searcher`, `mkn_diagnosis_getter`, `mkn_category_browser` per contracts/mcp-tools.md
- [x] T028 [US2] Add MKN CLI commands in `src/biomcp/cli/czech.py`: `mkn search --query`, `mkn get <code>`, `mkn browse [code]` with `--format json|human` output (Constitution IV)

**Checkpoint**: User Story 2 fully functional - MKN-10 search, detail, browse all working via MCP and CLI

---

## Phase 5: User Story 3 - Healthcare Provider Search (Priority: P3)

**Goal**: Physicians can search NRPZS providers by name/city/specialty, get full provider details with workplaces

**Independent Test**: Search "kardiologie Praha" returns matching providers, get provider by ID returns full record with workplaces

### Tests for User Story 3

> **NOTE: Write these tests FIRST, ensure they FAIL before implementation**

- [x] T029 [P] [US3] Write unit tests for NRPZS search in `tests/czech/test_nrpzs_search.py` - test search by city, specialty, name, combined filters, empty results, diacritics
- [x] T030 [P] [US3] Write unit tests for NRPZS getter in `tests/czech/test_nrpzs_getter.py` - test provider detail retrieval, workplaces, invalid ID error

### Implementation for User Story 3

- [x] T031 [P] [US3] Create Pydantic v2 models in `src/biomcp/czech/nrpzs/models.py`: `Address`, `Contact`, `Workplace`, `HealthcareProvider` per data-model.md
- [x] T032 [US3] Implement `_nrpzs_search()` and `_nrpzs_get()` in `src/biomcp/czech/nrpzs/search.py` using httpx against NRPZS API, with diacritics normalization, diskcache for response caching and offline fallback
- [x] T033 [US3] Register 2 NRPZS MCP tools in `src/biomcp/czech/czech_tools.py`: `nrpzs_provider_searcher`, `nrpzs_provider_getter` per contracts/mcp-tools.md
- [x] T034 [US3] Add NRPZS CLI commands in `src/biomcp/cli/czech.py`: `nrpzs search --query --city --specialty`, `nrpzs get <provider_id>` with `--format json|human` output (Constitution IV)
- [x] T035 [US3] Write integration test in `tests/czech_integration/test_nrpzs_api.py` marked `@pytest.mark.integration` - real HTTP calls to NRPZS API

**Checkpoint**: User Story 3 fully functional - provider search and details working via MCP and CLI

---

## Phase 6: User Story 4 - Health Procedure and Reimbursement Lookup (Priority: P4)

**Goal**: Users can search SZV procedures by code/name and look up VZP codebook entries for reimbursement info

**Independent Test**: Search "EKG" returns procedure with point value, get procedure "09513" returns full details, search VZP codebook returns entries

### Tests for User Story 4

> **NOTE: Write these tests FIRST, ensure they FAIL before implementation**

- [x] T036 [P] [US4] Write unit tests for SZV search/getter in `tests/czech/test_szv_search.py` and `tests/czech/test_szv_getter.py` - test procedure search by code/name, detail retrieval, point values, empty results
- [x] T037 [P] [US4] Write unit tests for VZP search/getter in `tests/czech/test_vzp_search.py` and `tests/czech/test_vzp_getter.py` - test codebook search, entry retrieval, codebook_type filter, invalid entry error

### Implementation for User Story 4

- [x] T038 [P] [US4] Create Pydantic v2 models in `src/biomcp/czech/szv/models.py`: `HealthProcedure`, `ProcedureSearchResult` per data-model.md
- [x] T039 [P] [US4] Create Pydantic v2 models in `src/biomcp/czech/vzp/models.py`: `CodebookEntry`, `CodebookSearchResult` per data-model.md
- [x] T040 [US4] Implement `_szv_search()` and `_szv_get()` in `src/biomcp/czech/szv/search.py` - use NZIP Open Data API v3 as primary, cache with diskcache
- [x] T041 [US4] Implement `_vzp_search()` and `_vzp_get()` in `src/biomcp/czech/vzp/search.py` - parse VZP codebook data, search by keyword, cache with diskcache
- [x] T042 [US4] Register 4 MCP tools in `src/biomcp/czech/czech_tools.py`: `szv_procedure_searcher`, `szv_procedure_getter`, `vzp_codebook_searcher`, `vzp_codebook_getter` per contracts/mcp-tools.md
- [x] T043 [US4] Add SZV and VZP CLI commands in `src/biomcp/cli/czech.py`: `szv search --query`, `szv get <code>`, `vzp search --query`, `vzp get <codebook_type> <code>` with `--format json|human` output (Constitution IV)
- [x] T044 [US4] Write integration test in `tests/czech_integration/test_szv_api.py` marked `@pytest.mark.integration` - real HTTP calls to szv.mzcr.cz

**Checkpoint**: User Story 4 fully functional - SZV procedure search/details and VZP codebook search/details working

---

## Phase 7: User Story 5 - Cross-Domain Clinical Query (Priority: P5)

**Goal**: All 14 Czech tools + 21+ global BioMCP tools coexist in single MCP session without interference

**Independent Test**: Start server, verify all tools registered, use SUKL tool then article_searcher in same session, verify both return valid results

- [x] T045 [US5] Verify all MCP tools register correctly: write test in `tests/czech/test_tool_registration.py` that imports `biomcp` and checks `mcp_app` has all 14 Czech tools plus existing global tools registered
- [x] T046 [US5] Write regression test in `tests/czech/test_no_regression.py` that verifies importing `biomcp.czech` does not break any existing module imports or tool registrations
- [x] T047 [US5] Import `czech_tools` module from `src/biomcp/czech/__init__.py` to ensure all Czech tools auto-register when the czech package is loaded

**Checkpoint**: All tools coexist - Czech and global tools work together in the same MCP session

---

## Phase 8: Polish & Cross-Cutting Concerns

**Purpose**: Documentation, cleanup, validation

- [x] T048 [P] Add Czech endpoint entries to `THIRD_PARTY_ENDPOINTS.md`: SUKL DLP API, NRPZS API, MKN-10 Data, NZIP Open Data, SZV Database, VZP Codebooks with URLs and auth info
- [x] T049 [P] Run `pytest tests/czech/ -v --cov=src/biomcp/czech --cov-report=term-missing` and verify >= 80% coverage (SC-005); also verify all models include `source` field for FR-015 source attribution
- [x] T050 Validate quickstart.md scenarios: run each CLI command from `specs/001-czech-health-sources/quickstart.md` and verify expected output format
- [x] T051 Run `ruff check src/biomcp/czech/` and `ruff format src/biomcp/czech/` to ensure code passes linting (line-length 79, target py310)
- [x] T052 [P] Write bilingual documentation for all 14 Czech tools: update `README.md` with Czech tools section (Czech + English), add `docs/czech-tools.md` with usage examples per tool (FR-016, SC-008, Constitution Dev Workflow)
- [x] T053 [P] Write performance benchmark test in `tests/czech/test_performance.py`: verify search latency < 2s cold / < 100ms cached (SC-001), ClaML parse < 5s (SC-007), MKN-10 accuracy >= 95% against sample code table (SC-004)

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Phase 1 completion - BLOCKS all user stories
- **User Stories (Phases 3-7)**: All depend on Phase 2 completion
  - US1 (Phase 3), US2 (Phase 4), US3 (Phase 5), US4 (Phase 6) can proceed in parallel
  - US5 (Phase 7) depends on US1-US4 all completing (validates cross-domain coexistence)
- **Polish (Phase 8)**: Depends on all user stories completing

### User Story Dependencies

- **User Story 1 (P1)**: Can start after Phase 2 - No dependencies on other stories
- **User Story 2 (P2)**: Can start after Phase 2 - Independent of US1 (uses local ClaML XML, not HTTP API)
- **User Story 3 (P3)**: Can start after Phase 2 - Independent of US1/US2
- **User Story 4 (P4)**: Can start after Phase 2 - Independent of US1/US2/US3
- **User Story 5 (P5)**: Depends on US1-US4 completing (integration/coexistence validation)

### Within Each User Story

- Tests MUST be written and FAIL before implementation
- Models before service functions
- Service functions before MCP tool registration
- MCP tools before CLI commands
- Integration tests last (require working implementation)

### Parallel Opportunities

- T003, T004, T005 can run in parallel (Phase 1 - different files)
- T007, T008, T009 can run in parallel (Phase 2 - different files)
- T012, T013, T014 can run in parallel (US1 tests - different files)
- T015 can run in parallel with tests (US1 model - different file)
- T022, T023 can run in parallel (US2 tests)
- T024 can run in parallel with tests (US2 model)
- T029, T030 can run in parallel (US3 tests)
- T036, T037 can run in parallel (US4 tests)
- T038, T039 can run in parallel (US4 models - different modules)
- Phases 3, 4, 5, 6 can run in parallel (independent user stories, different modules)

---

## Parallel Example: User Story 1

```bash
# Launch all US1 tests in parallel:
Task: "Write unit tests for SUKL drug search in tests/czech/test_sukl_search.py"
Task: "Write unit tests for SUKL drug getter in tests/czech/test_sukl_getter.py"
Task: "Write unit tests for SUKL availability in tests/czech/test_sukl_availability.py"

# Launch US1 model (can parallel with tests):
Task: "Create Pydantic v2 models in src/biomcp/czech/sukl/models.py"

# Then sequential: search.py -> getter.py -> availability.py -> czech_tools.py -> CLI -> integration test
```

## Parallel Example: Multiple User Stories

```bash
# After Phase 2 completes, launch all stories in parallel:
Agent A: Phase 3 (US1 - SUKL)
Agent B: Phase 4 (US2 - MKN-10)
Agent C: Phase 5 (US3 - NRPZS)
Agent D: Phase 6 (US4 - SZV + VZP)

# After all complete:
Phase 7 (US5 - Cross-Domain validation)
Phase 8 (Polish)
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational (CRITICAL - blocks all stories)
3. Complete Phase 3: User Story 1 (SUKL drug search)
4. **STOP and VALIDATE**: Test SUKL tools independently via MCP Inspector
5. Deploy/demo if ready - 5 Czech tools + 21+ global tools = immediate value

### Incremental Delivery

1. Setup + Foundational -> Foundation ready
2. Add US1 (SUKL) -> Test independently -> **MVP!** (5 Czech drug tools)
3. Add US2 (MKN-10) -> Test independently -> 8 Czech tools
4. Add US3 (NRPZS) -> Test independently -> 10 Czech tools
5. Add US4 (SZV+VZP) -> Test independently -> 14 Czech tools
6. US5 + Polish -> Cross-domain validation -> **v1.0 Release**

### Parallel Team Strategy

With multiple developers:

1. Team completes Setup + Foundational together
2. Once Foundational is done:
   - Developer A: US1 (SUKL - 5 tools)
   - Developer B: US2 (MKN-10 - 3 tools)
   - Developer C: US3 + US4 (NRPZS + SZV + VZP - 6 tools)
3. Stories complete and integrate independently
4. Final: US5 cross-domain validation + Polish

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- Each user story is independently completable and testable
- Tests use mocked HTTP responses (unit) and real API calls (integration, marked `@pytest.mark.integration`)
- All Czech code goes under `src/biomcp/czech/` - only 5 existing files modified (core.py, __init__.py, constants.py, domain_handlers.py, pyproject.toml)
- Follow existing BioMCP patterns: `@mcp_app.tool()`, `@track_performance()`, `Annotated[type, Field()]`
- CLI uses typer with `--format json|human` output (JSON default, human-readable per Constitution IV)
- Ruff config: line-length 79, target py310
