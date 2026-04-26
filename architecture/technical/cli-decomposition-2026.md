# CLI Decomposition 2026 Target State

## Context

BioMCP's CLI facade was decomposed once already, but several non-entity helper
surfaces and one test sidecar drifted back over the 700-line architecture cap.
The current problem is not the command grammar itself; it is that a small set of
flat files became catch-alls for planning, execution, formatting, local runtime
helpers, and tests.

This document records:

1. the current oversized CLI areas
2. the target module boundaries for each area
3. the child-ticket slice plan for reaching that state incrementally
4. the reusable no-public-surface-change proof contract

The enduring pattern is documented in
`architecture/technical/cli-module-decomposition.md`.

## Current problems

Survey ticket 309 identified four root issues:

1. **No CLI ratchet** equivalent to `tests/article_transform_structure.rs`
2. **Mixed responsibilities** inside large flat files
3. **Inline or flat test ownership** inflating runtime modules
4. **Missing reusable CLI decomposition doc** despite prior art in shipped tickets

The migration below addresses all four while keeping every intermediate state
`make check`-clean.

## Inventory

Seven files are directly in scope for the 2026 decomposition push:

| File | Lines | Current responsibility |
|---|---:|---|
| `src/cli/health.rs` | 3181 | Health source catalog, HTTP probes, local-data probes, orchestration, inline tests |
| `src/cli/search_all.rs` | 2984 | Cross-entity planning, dispatch, link generation, formatting/refinement, inline tests |
| `src/cli/suggest.rs` | 1654 | Offline question routing, entity extraction, regex catalog, inline tests |
| `src/cli/list.rs` | 1534 | Static CLI reference router plus 23 page builders and inline tests |
| `src/cli/article/tests.rs` | 1374 | Article CLI help/parse, exact-lookup, JSON integration, and filter-contract tests |
| `src/cli/benchmark/run.rs` | 1344 | Benchmark suite config, execution, regression analysis, report formatting, inline tests |
| `src/cli/skill.rs` | 1032 | Embedded skill catalog rendering plus install orchestration and inline tests |

**Coupled extra file:** `src/cli/benchmark/score.rs` is also over the cap at 824
lines. It should be decomposed in the same benchmark ticket because it shares the
same command family and would otherwise remain an over-cap outlier immediately
next to `run.rs`.

## Target architecture

### 1. `src/cli/health.rs` → `src/cli/health/`

**Stable facade:** `src/cli/health/mod.rs`

**Target files**

- `src/cli/health/mod.rs` — `HealthRow`, `HealthReport`, public `check()`, thin
  report assembly/re-exports
- `src/cli/health/catalog.rs` — `SourceDescriptor`, `ProbeKind`,
  `HEALTH_SOURCES`, `affects_for_api()`
- `src/cli/health/http.rs` — `send_request()` plus the HTTP probe helpers and
  latency formatting
- `src/cli/health/local.rs` — local-data, cache-dir, and cache-limit probes
- `src/cli/health/runner.rs` — probe execution order, timeout handling,
  buffered concurrency, `probe_source()`
- `src/cli/health/tests/{catalog,http,local,runner}.rs` — split sidecar tests

**Why this split**

The catalog is static data. HTTP probing is transport logic. Local-data and
cache checks are filesystem/runtime concerns. Concurrency and timeout handling
belong in a runner layer. Keeping those zones separate prevents future source
additions from touching unrelated transport and test code.

**Required invariants**

- `crate::cli::health::check()` remains the only public entry point
- rendered row order still follows the source catalog order
- every readiness-significant local source remains represented in `biomcp health`
- no API-key or error-message regressions on the health surface

**Ratchet after slice**

- `tests/health_cli_structure.rs`

### 2. `src/cli/search_all.rs` → `src/cli/search_all/`

**Stable facade:** `src/cli/search_all/mod.rs`

**Target files**

- `src/cli/search_all/mod.rs` — public types (`SearchAllInput`,
  `SearchAllSection`, `SearchAllResults`, `SearchAllLink`, `DispatchSpec`),
  `dispatch()`, `build_dispatch_plan()`, `counts_only_json()`
- `src/cli/search_all/plan.rs` — prepared input, section ordering, slot
  normalization, leg filters/routing, article filter construction
- `src/cli/search_all/dispatch.rs` — per-section execution, fetch limits,
  backfill merge helpers
- `src/cli/search_all/links.rs` — follow-up link generation, canonical search
  commands, filter hints, section-specific top-get commands
- `src/cli/search_all/format.rs` — JSON value formatting, drug-result
  refinement, variant ranking, GWAS dedupe helpers
- `src/cli/search_all/tests/{plan,dispatch,links,format}.rs` — split sidecar tests

**Why this split**

`search all` has a clear pipeline: prepare -> plan -> dispatch -> format
results and follow-ups. The render-layer dependency on
`crate::cli::search_all::SearchAllResults` stays stable because the facade keeps
that path alive.

**Required invariants**

- `crate::cli::search_all::SearchAllResults` remains importable at the current path
- section ordering, counts-only JSON behavior, and emitted follow-up commands do
  not drift
- no new clap/help/doc/spec surface changes

**Ratchet after slice**

- `tests/search_all_cli_structure.rs`

### 3. `src/cli/suggest.rs` → `src/cli/suggest/`

**Stable facade:** `src/cli/suggest/mod.rs`

**Target files**

- `src/cli/suggest/mod.rs` — `SuggestArgs`, `SuggestResponse`, public `run()`,
  `suggest_question()`, and test-only `route_examples()`
- `src/cli/suggest/routes.rs` — `ROUTES`, route summaries, and the 15 route
  matchers
- `src/cli/suggest/extract.rs` — entity extractors, anchor cleanup, topic
  normalization helpers
- `src/cli/suggest/patterns.rs` — `OnceLock<Regex>` factories, stopword lists,
  prefix constants
- `src/cli/suggest/tests/{routes,extract,render}.rs` — split sidecar tests

**Why this split**

The route table and route matchers are conceptually different from the regex
factory and from the anchor-extraction helpers. Keeping those separate makes
future route additions local and keeps the offline router understandable.

**Required invariants**

- `crate::cli::suggest::run()` and `suggest_question()` keep current behavior
- shipped route slugs and route-example parseability stay unchanged
- `suggest` remains fully offline and deterministic

**Ratchet after slice**

- `tests/suggest_cli_structure.rs`

### 4. `src/cli/list.rs` → `src/cli/list/`

**Stable facade:** `src/cli/list/mod.rs`

**Target files**

- `src/cli/list/mod.rs` — public `render()` and top-level router
- `src/cli/list/helpers.rs` — `all`, `discover`, `suggest`, `batch`, `enrich`,
  and `search_all` reference pages
- `src/cli/list/molecular.rs` — gene, variant, pathway, protein, pgx, and gwas
  reference pages
- `src/cli/list/clinical.rs` — disease, phenotype, diagnostic, trial, drug,
  and adverse-event reference pages
- `src/cli/list/literature.rs` — article and study reference pages
- `src/cli/list/tests/{router,pages}.rs` — split sidecar tests

**Why this split**

`list` is mostly static command-reference content. Grouping pages by surface area
keeps the router thin while avoiding another single giant asset-builder file.
This is intentionally conservative: it preserves the current output without
forcing a separate data-driven redesign.

**Required invariants**

- `crate::cli::list::render()` remains the only public entry point
- `biomcp list` output stays byte-stable aside from whitespace-neutral refactor
  noise that existing tests would catch
- no doc/help contract drift with README/docs CLI reference tests

**Ratchet after slice**

- `tests/list_cli_structure.rs`

### 5. `src/cli/article/tests.rs` → `src/cli/article/tests/`

**Stable facade:** `src/cli/article/tests.rs` shrinks to `mod` declarations only,
or is replaced by `src/cli/article/tests/mod.rs` if the Rust module layout makes
that cleaner.

**Target files**

- `src/cli/article/tests/mod.rs` — shared imports and module declarations
- `src/cli/article/tests/help.rs` — help-text and argument-parse tests
- `src/cli/article/tests/exact_lookup.rs` — exact entity lookup and suggestion tests
- `src/cli/article/tests/json.rs` — JSON integration and loop-breaker tests
- `src/cli/article/tests/filters.rs` — default-filter, ranking-context, and
  annotation-truncation tests

**Why this split**

This is already a sidecar, but it grew into four unrelated domains. Splitting
it lets article-runtime tickets touch only the relevant domain tests and keeps
integration fixtures separate from lightweight clap/help assertions.

**Required invariants**

- zero runtime behavior changes: this ticket is test-ownership only
- article CLI contract coverage remains the same or stronger
- wiremock-heavy JSON tests stay isolated from help/parser tests

**Ratchet after slice**

- `tests/article_cli_tests_structure.rs`

### 6. `src/cli/benchmark/run.rs` + `src/cli/benchmark/score.rs` → benchmark subtrees

**Stable facade:** `src/cli/benchmark/mod.rs` stays the command dispatcher.

**Target files for run path**

- `src/cli/benchmark/run/mod.rs` — `RunOptions`, `SaveBaselineOptions`,
  public `run_benchmark()`, `save_baseline()`
- `src/cli/benchmark/run/suite.rs` — suite constants, case specs, default
  thresholds, baseline path discovery
- `src/cli/benchmark/run/execute.rs` — command execution, temp cache setup,
  report collection, fail-fast handling
- `src/cli/benchmark/run/regression.rs` — baseline comparison and regression detection
- `src/cli/benchmark/run/render.rs` — human report rendering and formatting helpers

**Target files for score path**

- `src/cli/benchmark/score/mod.rs` — `ScoreSessionOptions`, public `score_session()`
- `src/cli/benchmark/score/parse.rs` — tool-call, token, and error extraction
- `src/cli/benchmark/score/normalize.rs` — command normalization and coverage helpers
- `src/cli/benchmark/score/render.rs` — markdown rendering helpers

**Why this split**

The benchmark family already has its own namespace. `run` and `score` are both
large but conceptually distinct: one executes benchmarks, the other scores agent
sessions. Keeping them in parallel subtrees fixes both over-cap files without
changing the top-level benchmark command grammar.

**Required invariants**

- `biomcp benchmark ...` command grammar remains unchanged
- the raw temp-dir exception currently allowed by
  `tests/test_central_test_support_helpers.py` remains valid unless the ticket
  deliberately absorbs it into a stronger shared helper
- baseline JSON schema and score-session report fields stay stable

**Ratchet after slice**

- `tests/benchmark_cli_structure.rs`

### 7. `src/cli/skill.rs` → `src/cli/skill/`

**Stable facade:** `src/cli/skill/mod.rs`

**Target files**

- `src/cli/skill/mod.rs` — `SkillCommand` and the stable public functions used
  by `src/cli/outcome.rs` and `src/mcp/shell.rs`
- `src/cli/skill/assets.rs` — embedded asset reads, canonical prompt loading,
  title/description parsing
- `src/cli/skill/catalog.rs` — use-case indexing, overview/list/show/render
  read-only operations
- `src/cli/skill/install.rs` — install path resolution, candidate scanning,
  confirmation, filesystem copy logic
- `src/cli/skill/tests/{catalog,install}.rs` — split sidecar tests

**Why this split**

Skill catalog rendering and skill installation are separate subsystems that only
share access to embedded assets. Keeping them separate also protects the MCP
resource boundary, which depends only on the read-only catalog surface.

**Required invariants**

- all current public functions remain at `crate::cli::skill::*`
- `mcp/shell.rs` resource rendering keeps working without import-path changes
- install behavior and read-only catalog behavior remain independently testable

**Ratchet after slice**

- `tests/skill_cli_structure.rs`

## Slice plan

All child tickets use `flow: build2`.

| Order | Slice | Scope | Survey issues | Proof focus |
|---|---|---|---|---|
| 1 | Search-all decomposition | `src/cli/search_all.rs` | 1, 2, 3 | stable `SearchAllResults` path + counts-only/help/spec behavior |
| 2 | Health decomposition | `src/cli/health.rs` | 1, 2, 3 | `biomcp health` output/help + local-source coverage |
| 3 | Suggest decomposition | `src/cli/suggest.rs` | 1, 2, 3 | routed-skill outputs and command parseability |
| 4 | Skill decomposition | `src/cli/skill.rs` | 1, 2, 3 | read-only skill resource surface + install workflow |
| 5 | List decomposition | `src/cli/list.rs` | 1, 2, 3 | `biomcp list` output and docs-contract tests |
| 6 | Article CLI test split | `src/cli/article/tests.rs` | 1, 3 | same article CLI coverage with isolated test domains |
| 7 | Benchmark cluster decomposition | `src/cli/benchmark/run.rs`, `src/cli/benchmark/score.rs` | 1, 2, 3 | benchmark help/output/schema stability |

This order front-loads the highest-risk public helper surfaces while leaving the
mostly internal benchmark and article-test cleanup until the public user-facing
surfaces are already under ratchet.

## Reusable no-public-surface-change proof

Every decomposition slice is a behavior-preserving refactor. Child tickets should
reuse this proof contract:

### Required gate

- `cargo test <focused area filter> --lib`
- `cargo clippy --lib --tests -- -D warnings`
- `make spec-pr`

### Surface canary rule

The ticket must preserve both:

1. `biomcp --help`
2. the `--help` output for the affected command family (`health`, `search all`,
   `suggest`, `list`, `skill`, or `benchmark`)

When an area already has executable spec or contract coverage, reuse that rather
than inventing a new snapshot. Examples:

- `health`, `list`, and `search all` → `spec/surface/cli.md`
- `suggest` and `skill` → `spec/surface/discover.md`
- `list` docs alignment → `tests/test_public_search_all_docs_contract.py` and
  `tests/test_public_skill_docs_contract.py` when relevant
- `benchmark` → focused long-help render tests plus existing benchmark unit tests

### Ratchet snippet child tickets can point to

Each slice should add an integration test shaped like:

```rust
#[test]
fn <area>_cli_files_stay_under_700_lines() {
    // walk src/cli/<area>/
    // assert every *.rs file starts with //!
    // assert every *.rs file is <= 700 lines
}
```

The test file should live in `tests/<area>_cli_structure.rs` so future work on
other CLI areas does not create merge churn in one shared ratchet file.

## Ticket-level success-checklist expectations

Every child ticket should say all of the following explicitly:

- which file or file cluster it decomposes
- which new submodules it introduces
- that the stable public module path stays unchanged
- what the intermediate state looks like after merge
- that the new area-specific structure ratchet lands in the same ticket
- which spec/contract/help probes prove no CLI surface drift

## Alignment with team goals and frontier

This work aligns with:

- **Goal G6: quality over features** — architecture ratchets and comprehensible
  review-sized files reduce regression risk without inventing new functionality
- **Frontier: Quality Ratchet Absorptions** — the unticketed CLI 700-line cap gap
  becomes a concrete sequence of build2 tickets
- **FAQ #18** — health decomposition must preserve coverage for every
  readiness-significant local source
- **FAQ #9** — list/help-facing tickets must keep docs-contract tests in the
  proof matrix when they touch the command-reference surface

No slice changes user-visible command grammar. This is architecture cleanup with
explicit surface-preservation proof.
