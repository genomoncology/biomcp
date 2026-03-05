# Dev Log — T015 Unified Multi-Source Article Search

## 2026-03-05

### Design adherence
- Implemented per `.climb/design.md` (AD1-AD9) without design deviations.
- No scope changes were introduced.

### Red-green TDD
1. Added failing tests first:
- `src/sources/pubtator.rs`
  - `entity_autocomplete_sets_expected_params`
  - `search_sets_expected_params_and_sort`
  - `search_includes_api_key_when_configured`
- `src/entities/article.rs`
  - `article_source_filter_parses_supported_values`
  - `planner_routes_all_to_europepmc_for_strict_filters`
  - `planner_rejects_pubtator_with_unsupported_strict_filters`
- `src/transform/article.rs`
  - `from_pubtator_search_result_maps_source_and_score`
- `src/render/markdown.rs`
  - `article_search_markdown_groups_results_by_source`
- `src/cli/mod.rs`
  - `search_article_parses_source_flag`

2. Confirmed red state (compile/test failures due missing new source/search API pieces).
3. Implemented code changes to satisfy tests.
4. Re-ran targeted tests and full suite to green.

### Implementation summary
- `src/sources/pubtator.rs`
  - Added `PubTatorClient::entity_autocomplete(query)` using `/entity/autocomplete/`.
  - Added `PubTatorClient::search(text, page, size, sort)` using `/search/`.
  - Added response structs:
    - `PubTatorAutocompleteResult`
    - `PubTatorSearchResponse`
    - `PubTatorSearchResult`
  - Preserved existing auth/error/content-type handling and API-key forwarding.

- `src/entities/article.rs`
  - Added `ArticleSource` metadata and `ArticleSourceFilter` (`all|pubtator|europepmc`).
  - Extended `ArticleSearchResult` with `source` and `score`.
  - Refactored `search_page` into source-aware orchestrator:
    - Planner rules via `plan_backends(...)`.
    - Single-source legs: `search_europepmc_page(...)`, `search_pubtator_page(...)`.
    - Federated leg: `search_federated_page(...)` using `tokio::join!`.
  - Implemented graceful degradation:
    - One backend failure returns surviving backend.
    - Both failures return error.
  - Implemented PMID dedup with PubTator priority in federated merge.
  - Added PubTator entity normalization via autocomplete before query construction.
  - Implemented fixed-page-size offset handling:
    - Europe PMC: 25
    - PubTator: 25
  - Preserved strict filter correctness rules:
    - `--open-access` and `--type` are Europe-only strict filters.
    - `--source pubtator` with strict filters returns `InvalidArgument`.
    - `--source all` routes to Europe-only when strict filters are present.

- `src/transform/article.rs`
  - Added `from_pubtator_search_result(...)`.
  - Updated Europe PMC mapping to set `source=EuropePmc`.

- `src/cli/mod.rs`
  - Added `search article --source <all|pubtator|europepmc>` CLI flag.
  - Parsed and propagated source filter into `article::search_page(..., source_filter)`.
  - Included non-default source in query summary.

- `src/cli/search_all.rs`
  - Updated article section to use source-aware search (`ArticleSourceFilter::All`).
  - Changed article sort to `relevance` for entity-ranked behavior.

- Rendering/docs
  - `src/render/markdown.rs` + `templates/article_search.md.j2`:
    - Group article results by source.
    - Render source-specific columns (`Score` for PubTator, `Cit.` for Europe PMC).
  - `src/cli/list.rs`, `src/cli/list_reference.md`:
    - Added `--source <all|pubtator|europepmc>` documentation.

### Quality gates
- `cargo fmt` ✅
- `cargo build` ✅
- `cargo test` ✅ (383 passed)
- `cargo clippy -- -D warnings` ✅
- `make spec` not run (no spec files/target present in repo)

### Challenges / notes
- PubTator coerces small `size` values; fixed internal page size (`25`) avoids offset drift.
- Federated totals are not exact under cross-source dedup; federated response uses `total=None`.
