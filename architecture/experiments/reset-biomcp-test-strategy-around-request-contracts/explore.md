# Explore — Reset BioMCP test strategy around request contracts

## Spike Question

How should BioMCP reset its routine development and March validation strategy around deterministic request/plan contracts so ordinary tickets are not blocked by slow or flaky live upstream services?

## Prior Art Summary

BioMCP already has strong pieces of the desired shape, but they are not the routine gate's organizing principle yet.

- March validation profile routing existed when this experiment was written, but the repo now uses the standard `make lint`, `make test`, and `make spec` gates directly.
- `Makefile` makes `release-gate = check spec-pr`; `spec-pr` builds a release binary, syncs Python dev deps, runs most executable specs in xdist, then serializes `spec/entity/protein.md`, `spec/entity/disease.md`, and `spec/surface/discover.md`.
- `tools/biomcp-ci` is useful prior art: it centralizes executable-spec isolation, key stripping, cache roots, and warm-cache replay. Keep it for retained executable specs and live smoke.
- There is no general `RequestCommand` today. CLI dispatchers usually convert clap args to entity filter structs and immediately execute entity/source calls.
- Local plan seams do exist and should be reused: `src/cli/search_all/plan.rs`, `src/entities/article/planner.rs`, and `src/cli/debug_plan.rs`.
- Source modules already contain strong deterministic request-contract tests using `new_for_test`, `env_base`, and wiremock. Representative files (`ols4`, `mydisease`, `myvariant`, `pubmed`, `europepmc`, `semantic_scholar`) contain 40 `Mock::given` contracts and 101 query-param assertions in the spike inventory.

Recent March evidence from ticket 370 shows why the status quo is the wrong routine gate:

- `make spec-pr` failed twice before the HGVS-normalization work could start because unrelated serialized OLS4/disease/discover specs failed.
- Failures were `spec/entity/disease.md::Synonym Rescue` twice and `spec/surface/discover.md::MEF2 relational query` once.
- In the eventual green run, the main spec leg passed 127 items in 179.32s and the serialized partition passed 22 items in 49.20s, after release build/Python setup. Earlier failed main legs took 88.31s and 130.41s.

Raw evidence: `architecture/experiments/reset-biomcp-test-strategy-around-request-contracts/results/test_strategy_inventory.json`.

## Approaches Tried

### 1. Status quo with narrower live partitioning

**What:** Keep the current spec-v2 executable-doc model, but move the known OLS4-heavy serialized partition out of kickoff/focused gates.

**How measured:** Static profile/Makefile inventory plus ticket-370 preflight evidence extraction.

**Measurements:**

- 3 ticket-370 preflight runs inspected.
- 2/3 failed due to unrelated live-source/spec partition failures.
- Failed sections: disease Synonym Rescue twice; MEF2 discover relational query once.
- The current routine `spec-pr` shape still requires release build, Python setup, fixture setup, 127-item xdist leg, and 22-item serialized live-heavy leg.

**Result:** Better than running everything in parallel, but not sufficient. It still makes routine work prove OLS4/disease/discover live availability.

### 2. Contract-first pyramid using existing seams

**What:** Treat deterministic contracts as the routine gate: CLI args to typed filters/request plan; source request construction via wiremock/fixtures; response/status mapping; renderer/JSON envelope tests; tiny smoke only when explicitly requested.

**How measured:** Static representative sample over `spec/entity/disease.md`, `spec/surface/discover.md`, `spec/entity/article.md`, and `spec/entity/variant.md`, plus source-test inventory.

**Measurements:**

- 39 representative spec sections classified.
- 22 sections used the live `tools/biomcp-ci` wrapper; 19 were live and non-fixture-backed.
- Only 3 representative sections were fixture-backed.
- Proof tags showed 38 CLI routing/rendering checks but only 8 source-request-construction and 7 command/request-plan checks.
- Source prior art is strong: 40 wiremock `Mock::given` blocks, 47 async source tests, 101 query-param assertions, 8 header assertions, 7 body assertions.

**Result:** Winner. The repo already knows how to test request construction deterministically; the strategy should promote that layer and shrink routine executable specs.

### 3. Minimal request-plan seam introduction

**What:** Design a small seam without changing product behavior in the spike: command payloads expose a request/plan object that tests can assert before execution.

**How measured:** Mapped representative dispatchers and existing plan structs.

**Measurements:**

- Existing explicit plan functions found in `src/cli/search_all/plan.rs`, `src/entities/article/planner.rs`, and `src/cli/article/dispatch.rs`.
- Existing candidate structs/enums include `PreparedInput`, `DispatchSpec`, `DebugPlan`, `DebugPlanLeg`, `BackendPlan`, `ArticleSearchFilters`, `DiseaseSearchFilters`, and variant search filter/request structs.
- Gaps: disease/discover/variant dispatch still commonly constructs filters and executes immediately; no general source request-plan representation for method/url/query/header/body/cache/auth.

**Result:** Promote an incremental seam, not a repo-wide abstraction rewrite. Start with hot spots and make each source/command testable before network execution.

## Decision

Use the **contract-first pyramid with an incremental request-plan seam**.

Routine March gates should stop treating broad live executable specs as kickoff/focused proof. The new pyramid should be:

1. **CLI intent layer** — clap args and command dispatch produce typed `RequestCommand`/filter/plan values. Proves parsing, defaults, validation, and routing without network.
2. **Source request layer** — source clients expose or build `SourceRequestPlan` values covering method, URL/path, query, headers, body, auth mode, and cache mode; wiremock/fixtures verify response and status mapping.
3. **Entity orchestration layer** — entity code combines source plans/results and degrades truthfully. Proves fallback, optional enrichment, pagination, and source capability semantics.
4. **Renderer/envelope layer** — markdown and JSON `_meta`/`next_commands`/provenance shape from fixture models. Proves agent-facing contracts without live services.
5. **Small executable smoke layer** — a tiny opt-in live lane for release/operator confidence only.

### Request seam decision

Introduce a request seam, but incrementally:

- `RequestCommand` belongs at CLI/entity boundary: it captures user intent after clap normalization and validation.
- `SourceRequestPlan` belongs inside each source module or a shared `sources::request_plan` helper: it captures method, URL/path, query, headers, body, auth mode, and cache mode.
- Runtime clients may still execute with reqwest; tests should assert plans before execution and use fixtures/wiremock for mapping.
- Do not force every command into one monolithic enum before value is proven. Start with disease/discover, article, and variant because they are current pain points.

### Proposed March profile map

| Profile | Proposed behavior | Live network? | What it proves |
|---|---|---:|---|
| kickoff/preflight | `cargo check --all-targets` plus touched-area deterministic contract tests when available | No | Compilation, basic command/request contract health |
| focused | `cargo test --lib`/targeted module tests + clippy; include touched source wiremock tests | No | Unit behavior, request construction, fixture mapping |
| spec-only | Fixture-backed executable specs and static surface contracts only | No by default | CLI help/list, JSON envelope, renderer contracts |
| verify/full-blocking | `make check` plus deterministic spec-only; broad live canaries excluded unless explicitly requested | Minimal/controlled | Repo health and quality ratchets |
| release/live-smoke | Small serialized opt-in live smoke for OLS4/discover, disease crosswalk, article sources, variant normalization | Yes | Public upstream availability and release confidence |

## Keep / Move / Mock / Delete Matrix

| Area / representative specs | Current proof | Decision | Rationale |
|---|---|---|---|
| `spec/surface/discover.md::Alias-Like Free Text` | OLS4 live alias resolution | Mock + one release smoke | Request to OLS4 and result mapping can be deterministic; live OLS4 belongs in smoke. |
| `spec/surface/discover.md::MEF2 relational query` | Live OLS4 timeout-prone redirect behavior | Move to fixture-backed contract; keep tiny live smoke for OLS4 health | This blocked ticket 370 despite being unrelated. The behavior is routing/guardrail, not a per-ticket upstream canary. |
| `spec/entity/disease.md::Synonym Rescue` | Discover + crosswalk through OLS4/MyDisease | Mock/fixture; live release smoke optional | This failed twice in ticket 370. Contract should assert crosswalk plan and fixture response mapping. |
| `spec/entity/disease.md::Genes & Diagnostics` | Live disease/GTR/OpenTargets style enrichment | Split: deterministic entity/render tests; optional smoke for upstream readiness | Routine gate should prove section gating/rendering and source requests, not all upstream availability. |
| `spec/entity/article.md::Gene Search` | Live federated article search | Mock source requests and response mapping; keep one release smoke | Article planner already has `BackendPlan`; assert plan and fixture merge/ranking. |
| `spec/entity/article.md::Full-Text HTML/PDF Fallback` | Fixture-backed source server | Keep | This is the right pattern: deterministic upstream shape with user-visible fallback proof. |
| `spec/entity/article.md::Semantic Scholar status` | Live/no-key S2 behavior | Mock status mapping; release smoke optional | Auth/cache/status redaction should be deterministic; live S2 shared-pool rate limits are not routine proof. |
| `spec/entity/variant.md::Gene-Scoped Search` | Live MyVariant search | Mock source request and fixture rows; keep one smoke anchor | Query construction and rendering are deterministic contracts. |
| `spec/entity/variant.md::ID Normalization` | Live MyVariant exact lookup | Mock fixture for routine; release smoke optional | Routine proof is canonicalization behavior; public service availability is separate. |
| `spec/entity/variant.md::variant normalize all` | Live Mutalyzer/VariantValidator | Move to release/live smoke plus fixture-backed parser/status mapping | Ticket 370 was about this feature but got blocked by unrelated gates; normalization services are exactly the kind of live canary to isolate. |
| Pure help/list/structure specs | Static CLI/docs contracts | Keep in routine | Fast, deterministic, high signal. |
| Redundant exact prose/format assertions | Brittle copy/format pins | Delete or relax | Preserve behavior checks; avoid exact JSON/prose pins that do not catch product regressions. |

## Follow-up Ticket Set

1. **Build: add request-plan primitives and first source-plan tests for OLS4/MyDisease** — introduce `SourceRequestPlan` or equivalent in source modules; convert discover/disease synonym/crosswalk contracts to fixture-backed tests.
2. **Build: add CLI `RequestCommand` seams for disease/discover and article** — expose normalized intent/filter/plan values and assert them without network.
3. **Quickfix: split March validation profiles** — make kickoff/focused deterministic only; add explicit `release-live-smoke` profile and remove OLS4/disease/discover from routine spec-only.
4. **Build: convert representative article and variant specs to fixture-backed contracts** — mock PubMed/EuropePMC/S2/MyVariant/normalization service mappings for routine proof.
5. **Review/quickfix: prune redundant brittle executable specs** — delete or relax exact prose/count assertions once deterministic contract tests cover the behavior.

## Outcome

promote

## Risks for Exploit

- A too-large universal `RequestCommand` enum could stall implementation. Keep the seam incremental and source/entity-local at first.
- Fixture tests can drift from real upstreams. Mitigate with the small explicit release/live-smoke lane and periodic fixture refresh when upstream contracts change.
- Moving live specs too quickly could hide real release-readiness regressions. Move only after deterministic replacement tests exist.
- March profile names must stay compatible with current flow expectations; change profile behavior deliberately and document the live-smoke opt-in.
- Some current specs prove combined user workflows, not just one request. Preserve a few cross-entity workflow smoke tests with fixtures so agent-facing behavior remains covered.

## No Runtime Behavior Change

This spike changed no BioMCP runtime product behavior. It added reproducible experiment scripts/results, removed tracked `.march/` state from git, and documents the proposed strategy.
