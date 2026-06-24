# Request-Contract Test Architecture Target

Ticket 373 records the repo architecture target for the request-contract testing reset accepted in ticket 371. The strategy artifact is the source input: `planning/biomcp/artifacts/371-reset-biomcp-test-strategy-around-request-contracts/request-contract-test-strategy.md`.

This document describes a target state, not the current implementation. It deliberately preserves BioMCP runtime behavior and changes only the seams and gates needed to make routine tests deterministic.

## Problems being corrected

The survey for ticket 373 identified six root causes:

1. At the time of the ticket-373 survey, routine March validation still routed ordinary proof through one live/cache-backed executable-spec canary lane, while `Makefile` kept disease/discover/protein serialized canaries inside `spec` and `spec-pr`. Ticket 378 split routine proof onto deterministic `make spec-contracts` and left public upstream checks in explicit `make release-live-smoke`.
2. Disease and discover jump from CLI args to network-backed entity execution without a durable request-command seam: `src/cli/discover.rs::run()`, `src/entities/discover.rs::resolve_query()`, `src/cli/disease/dispatch.rs::handle_search()`, `src/entities/disease/search.rs::search_page()`, and `src/entities/disease/fallback.rs::fallback_search_page()` interleave intent, fallback, and source execution.
3. Source request construction is trapped inside execution methods such as `src/sources/ols4.rs::OlsClient::search()`, `src/sources/mydisease.rs::{query,lookup_disease_by_xref,get}()`, `src/sources/myvariant.rs::{query_with_fields,search,get}()`, `src/sources/pubmed.rs::esearch()`, `src/sources/europepmc.rs::search_query_with_sort()`, and `src/sources/semantic_scholar.rs::paper_search()`.
4. Existing planning seams are uneven: article has `src/entities/article/planner.rs::BackendPlan`, variant has `src/cli/variant/dispatch.rs::VariantSearchPlan`, and search-all has `src/cli/search_all/plan.rs`, but none of these is a complete source-request contract and disease/discover have no equivalent request seam.
5. Executable specs still conflate BioMCP behavior with public upstream availability. Ticket 372 quarantined the worst disease/discover blockers as prose, leaving behavior to restore through fixture-backed or explicit live-smoke coverage.
6. Renderer/envelope and next-command ownership leaks across boundaries. CLI dispatchers, render modules, and entities each contribute output guidance, so deterministic replacement tests need clear result-model and envelope ownership before live specs are pruned.

## Target boundaries

The target architecture has three deterministic boundaries before any live smoke runs.

```text
clap args
  -> entity-local Request values
  -> entity/source orchestration plans
  -> source-local request plans
  -> fixture or wiremock response/status mapping
  -> entity result models
  -> renderer/envelope contracts
  -> optional explicit release-live-smoke
```

### 1. CLI/entity request boundary

Use entity-local request values rather than a repository-wide monolithic enum.

First-wave values:

- `src/cli/discover.rs` / `src/entities/discover.rs`: introduce a test-visible `DiscoverRequest` builder for `DiscoverArgs`. It owns normalized free text, mode selection (`DiscoverMode::Command` versus `DiscoverMode::AliasFallback`), and flags needed to decide whether the request is an OLS4 disease/symptom lookup, an alias-like query, or redirect guidance. `run()` should build the request, pass it to execution, and render as it does today.
- `src/cli/disease/dispatch.rs` / `src/entities/disease/search.rs` / `src/entities/disease/fallback.rs`: introduce `DiseaseSearchRequest` and a small `DiseaseFallbackRequest`/`DiseaseFallbackPlan` that capture the normalized query, `DiseaseSearchFilters`, limit, HPO field filter, and synonym-rescue/crosswalk intent before `MyDiseaseClient` or `resolve_query()` is constructed.
- `src/cli/article/dispatch.rs` / `src/entities/article/planner.rs`: keep `ArticleSearchFilters` and `BackendPlan`, but add a request-command wrapper that records source/ranking/sort flags and the pre-execution `BackendPlan`. CLI debug-plan output remains post-execution, but tests can assert the pre-execution request value.
- `src/cli/variant/dispatch.rs`: keep `VariantSearchRequest`, `ResolvedVariantQuery`, and `VariantSearchPlan` as prior art. Later source-plan work should connect the resolved query to MyVariant request plans instead of adding a second CLI planning type.

Invariants:

- Clap/default normalization and validation can be asserted without constructing source clients.
- Request values do not hold `reqwest` clients, cache handles, base URLs, or response bodies.
- Request values are entity-local and small enough to keep one public behavior zone per ticket.
- Runtime control flow remains the same after the request is executed; user-facing CLI/MCP behavior does not change.

### 2. Source request-plan boundary

Add source-local request plan builders first. Do not start with a shared `sources::request_plan` abstraction unless two shipped source-local implementations prove enough common shape to extract it safely.

First-wave source-local plan types should cover:

- `src/sources/ols4.rs`: `OlsSearchRequestPlan` or equivalent builder for the `/api/search` request currently constructed in `OlsClient::search()`. It must expose method, path, canonical query parameters (`q`, `rows`, `groupField`, ontology list), base/source label, timeout/cache intent if source-specific, and no secret-bearing fields.
- `src/sources/mydisease.rs`: separate request plan builders for `query`, `lookup_disease_by_xref`, and `get`. Plans must expose method/path/query parameters, selected fields, normalized xref/name lookup input, cache/no-cache mode, and status mapping expectations.
- Later source-local plans: `src/sources/myvariant.rs` for search/get/query-with-fields; `src/sources/pubmed.rs::esearch()`/`esummary()`; `src/sources/europepmc.rs::search_query_with_sort()`; `src/sources/pubtator.rs`; `src/sources/litsense2.rs`; `src/sources/semantic_scholar.rs::paper_search()` and graph helpers; `src/sources/mutalyzer.rs`; `src/sources/variantvalidator.rs`.

Execution methods may keep their public names. They should call the local plan builder, execute the plan through the existing client/cache path, read bounded bodies, and map status/content type exactly as they do today.

Invariants:

- Request construction is asserted before network execution.
- Wiremock/fixture tests assert response and status mapping against the plan, not duplicated query-building logic.
- Auth mode and header redaction are testable without asserting literal secret values.
- Cache/live behavior is explicit in the plan or test harness; routine tests do not depend on ambient public upstream availability.

### 3. Entity orchestration and renderer/envelope boundary

Entity orchestration tests should use request values, source request plans, and fixture source results to prove fallback and merge behavior. Renderer tests should use fixture result models to prove markdown/JSON envelopes.

Target ownership:

- Entities own semantic results, source status, fallback/degradation decisions, pagination, provenance, and typed next-command data.
- Render modules own markdown/JSON serialization and shell-safe presentation of next commands.
- CLI dispatchers own argument parsing, request construction, execution call, and selecting markdown versus JSON output.

First cleanup should not move all next-command construction at once. Instead, tests should pin the desired boundary by adding fixture result builders for disease/discover/article/variant and asserting `_meta.next_commands`, `_meta.source_status`, provenance, and markdown table/card anchors without live calls.

Invariants:

- Entity code should not depend on markdown-specific quoting helpers for semantic command construction in new or touched request-contract paths.
- JSON `_meta.next_commands` is asserted as a behavior contract, not as incidental text.
- Markdown tests assert stable structural anchors, not exact prose/count trivia unless the exact text is the product contract.

Current next-command ownership debt: older entity/render paths still mix semantic
follow-up construction with markdown presentation. Treat that as a named
next-command ownership follow-up rather than widening this ticket into a broad
renderer refactor.

## Retired validation profile notes

March validation profiles are retired. The repository no longer keeps a tracked
March profile file, and routine proof routing is expressed by the standard repo
gates instead:

- `make lint`
- `make test`
- `make spec`

Keep live public-upstream confidence explicit and opt-in through `make verify`
(with `make release-live-smoke` as a compatibility alias). Do not bury live calls
inside routine local proof.

Current invariants:

- Dependency-free build tickets can still run `make lint`, `make test`, and
  `make spec` successfully at every intermediate state.
- No live public API call is required by ordinary routine proof.
- `tools/biomcp-ci` remains the executable-spec wrapper for cache roots, XDG
  roots, key stripping, and warm replay in live/operator lanes.
- `.march/code-review-log.md` is the only tracked `.march/*` allowlist entry;
  other local March artifacts stay ignored and untracked.

## Coverage restoration and pruning order

Do not prune live executable assertions just because they are flaky. Remove or relax them only after their replacement contract exists or the behavior is deliberately classified as release-live-smoke-only.

Ordered restoration targets:

1. Restore the ticket-372 disease/discover holes first:
   - `spec/entity/disease.md::Synonym Rescue`
   - `spec/surface/discover.md::MEF2 relational query`
   These become deterministic source-plan/request-command/fixture contracts plus optional release-live-smoke anchors.
2. Convert article and variant representative live sections:
   - article gene/keyword search, Semantic Scholar keyless/status behavior, and fulltext fixture-preserving paths;
   - variant MyVariant search/get, ID normalization, and Mutalyzer/VariantValidator normalization status.
3. Add renderer/envelope fixture contracts for `_meta.next_commands`, `_meta.source_status`, provenance, and markdown/JSON shape.
4. Split profiles only when the deterministic routine gate proves the same user-visible contracts without live upstream dependency.
5. Prune exact prose/count/live assertions after replacements land, applying the spec-v2 semantic/structural/trivia rubric.

## Blast-radius checks for implementation tickets

Every implementation ticket that touches these areas should include the relevant proof commands in its success checklist:

- `cargo test --lib` for Rust request builders, source plans, entity orchestration, and renderer model tests.
- Targeted module tests where useful, e.g. `cargo test --lib sources::ols4`, `cargo test --lib sources::mydisease`, `cargo test --lib entities::article::planner`, `cargo test --lib cli::variant`.
- `uv run --no-sync pytest spec/surface/test_parallel_isolation_contract.py -v` when changing `Makefile`, OLS4 disease/discover spec partitioning, or the ticket-372 quarantine replacement.
- `make lint`, `make test`, and `make spec` as the canonical local gates, preserving FAQ #12's advisory visibility.
- Deterministic spec commands (`make spec-contracts` for routine proof, with `make spec-pr` retained for full canary debugging) only after replacement fixture contracts exist.

## Assumptions to validate during rollout

- Source-local plan types for OLS4 and MyDisease will expose enough common fields to decide whether a shared helper is useful. If not, keep source-local types and avoid abstraction churn.
- Fixture-backed disease/discover and article/variant replacements can preserve user-visible behavior without exact live upstream rows. If a section cannot be expressed deterministically, classify it as explicit release-live-smoke instead of forcing it into routine gates.
- The final deterministic spec/profile split will still keep local `make lint`, `make test`, and `make spec` under release-acceptable time and preserve the advisory/lint contracts. If not, keep `spec-only` narrow and leave heavier live checks in release-live-smoke.
