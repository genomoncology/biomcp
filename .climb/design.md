# T015: Unified Multi-Source Article Search - Design (Reviewed/Corrected)

## Summary

Implement federated article search across PubTator3 + Europe PMC, with a new
`--source` selector and PMID deduplication. Keep existing Europe PMC behavior
intact for filters PubTator cannot enforce exactly.

## Verified Baseline (Current Code)

- `ArticleSearchResult` currently has no source/score metadata (`src/entities/article.rs:70`).
- `search_page(filters, limit, offset)` is Europe PMC-only (`src/entities/article.rs:525`).
- PubTator client currently exposes only `export_biocjson()` (`src/sources/pubtator.rs:82`).
- CLI `search article` currently has no `--source` flag (`src/cli/mod.rs:345`).
- `search all` article section currently calls Europe PMC-only `search_page()` (`src/cli/search_all.rs:684`).

## External API Facts Validated for This Design

- PubTator autocomplete endpoint returns an array with fields including
  `_id`, `biotype`, `db_id`, `name`.
- PubTator search endpoint returns keys:
  `results`, `count`, `total_pages`, `current`, `page_size`, `facets`.
- PubTator `size` values `<10` are coerced to `10` in responses, so pagination
  math must be based on an explicit internal page size, not user `limit`.

## Architecture Decisions

### AD1: Add explicit article source metadata

Extend `ArticleSearchResult` with:
- `source: ArticleSource` (`PubTator`, `EuropePmc`)
- `score: Option<f64>` (PubTator relevance score)

This enables dedup-first merge logic and source-aware rendering.

### AD2: Add source filter enum + CLI `--source`

Add `ArticleSourceFilter` with:
- `all` (default)
- `pubtator`
- `europepmc`

Wire it into `SearchEntity::Article` in `src/cli/mod.rs` and pass through to
`article::search_page(...)`.

### AD3: PubTator search + autocomplete client methods

Add to `PubTatorClient`:
- `entity_autocomplete(query)`
- `search(text, page, size, sort)`

Add response structs:
- `PubTatorAutocompleteResult`
- `PubTatorSearchResponse`
- `PubTatorSearchResult`

Use existing shared request behavior:
- `append_ncbi_api_key(...)`
- `get_json(...)` with current error handling/content-type checks.

### AD4: Backend planner to preserve filter correctness

Federated mode is default, but we must not silently violate strict filters.

Planner behavior:
- `source=europepmc`: Europe PMC only.
- `source=pubtator`: PubTator only; reject unsupported strict filters with
  `InvalidArgument` (`--open-access`, `--type`).
- `source=all`: use both unless strict Europe-only filters are present
  (`--open-access`, `--type`), in which case route to Europe PMC only.

This preserves existing semantics while still improving common entity searches.

### AD5: Pagination model (corrected)

Use fixed backend page sizes (not `limit`) for offset math:
- `EPMC_PAGE_SIZE = 25` (existing behavior)
- `PUBTATOR_PAGE_SIZE = 25` (>=10 to avoid PubTator size coercion edge)

Single-source pagination:
- `page = (offset / PAGE_SIZE) + 1`
- `local_skip = offset % PAGE_SIZE`
- iterate pages until `limit` rows collected or backend exhausted.

Federated pagination:
- run both backends concurrently with the same `(limit, offset)` window.
- merge PubTator first, then Europe PMC, dedup by PMID, truncate to `limit`.
- `total` in federated responses is `None` (combined total is not exact under dedup).

### AD6: Search orchestration and failure handling

`search_page(filters, limit, offset, source_filter)` becomes orchestrator:
- `All` -> `tokio::join!` PubTator + Europe PMC legs (per planner)
- single-source -> one backend leg

Failure behavior:
- if one leg fails in federated mode, return successful leg results
- if both fail, return first error

### AD7: Entity normalization

Before PubTator search, normalize `gene/disease/drug` via autocomplete
(best-effort):
- prefer canonical `_id` when `biotype` matches expected class
- fall back to raw user token when autocomplete fails/no-match

Build PubTator query from normalized IDs + keyword token.

### AD8: search-all integration

Update article dispatch in `src/cli/search_all.rs` to call new
`search_page(..., ArticleSourceFilter::All)` and set `sort=Relevance` for the
article section. This uses entity-ranked PubTator results first while keeping
Europe PMC fallback behavior.

### AD9: Keep non-search article flows unchanged

- `get article` path remains unchanged.
- MCP shell behavior remains unchanged (delegates to CLI).

## File Disposition

### Modify

| File | Changes |
|------|---------|
| `src/sources/pubtator.rs` | Add `search()` + `entity_autocomplete()` and new response structs for autocomplete/search payloads. |
| `src/entities/article.rs` | Add `ArticleSource` + `ArticleSourceFilter`; extend `ArticleSearchResult` with `source`/`score`; refactor `search_page()` into source-aware orchestrator; add PubTator leg + dedup merge + planner; keep `get()` flow intact. |
| `src/transform/article.rs` | Add `from_pubtator_search_result()` transform. |
| `src/cli/mod.rs` | Add `--source` arg to `SearchEntity::Article`; parse to `ArticleSourceFilter`; include source in query summary; pass source into `search_page()`. |
| `src/cli/search_all.rs` | Switch article section to new `search_page(..., source)` call and `ArticleSort::Relevance`. |
| `templates/article_search.md.j2` | Render source-aware sections/table(s) and include `--source` help text. |
| `src/render/markdown.rs` | Add source-aware render path for article search output (grouped display). |
| `src/cli/list.rs` | Update `list article` help to document `--source <all|pubtator|europepmc>`. |
| `src/cli/list_reference.md` | Add `search article ... --source <all|pubtator|europepmc>` to quick reference. |

### No Changes

| File | Reason |
|------|--------|
| `src/sources/europepmc.rs` | Existing search client/sort support is sufficient. |
| `src/mcp/` | CLI delegation means no MCP routing changes required. |
| `src/entities/article.rs` get-path internals | `get()` and full-text/annotation retrieval are out of scope. |

## Key Sketches

### Source-aware `search_page`

```rust
pub async fn search_page(
    filters: &ArticleSearchFilters,
    limit: usize,
    offset: usize,
    source: ArticleSourceFilter,
) -> Result<SearchPage<ArticleSearchResult>, BioMcpError> {
    match plan_backends(filters, source)? {
        BackendPlan::EuropeOnly => search_europepmc(filters, limit, offset).await,
        BackendPlan::PubTatorOnly => search_pubtator(filters, limit, offset).await,
        BackendPlan::Both => search_federated(filters, limit, offset).await,
    }
}
```

### PubTator sort mapping

```rust
fn pubtator_sort(sort: ArticleSort) -> Option<&'static str> {
    match sort {
        ArticleSort::Date => Some("date desc"),
        ArticleSort::Relevance => None,
        ArticleSort::Citations => None, // fallback to relevance
    }
}
```

## Acceptance Criteria

### AC1: PubTator endpoints wired
- `PubTatorClient::entity_autocomplete()` calls `/entity/autocomplete/` with `query`.
- `PubTatorClient::search()` calls `/search/` with `text/page/size/(optional)sort`.
- Both methods forward `api_key` when `NCBI_API_KEY` is set.

### AC2: Source-aware article model
- `ArticleSearchResult` includes `source` and optional `score`.
- Europe PMC transform sets `source=EuropePmc`.
- PubTator transform sets `source=PubTator` and carries `score`.

### AC3: CLI/source plumbing complete
- `search article` supports `--source <all|pubtator|europepmc>`.
- Parsed source value is passed into `article::search_page(...)`.
- Query summary/footer includes source selection when non-default.

### AC4: Federated parallel search + graceful degradation
- `source=all` executes PubTator and Europe PMC concurrently (`tokio::join!`).
- PMID dedup keeps first hit (PubTator priority).
- If one backend fails, return the other backend's results.
- If both fail, return an error.

### AC5: Pagination correctness
- Offset logic for PubTator uses fixed page size, not `limit`.
- `--offset` works for single-source modes.
- Federated mode returns deterministic merged windows and `total=None`.

### AC6: Filter compatibility safeguards
- `--open-access` and `--type` remain exact via Europe PMC.
- `source=pubtator` with those strict filters returns `InvalidArgument`.
- `source=all` auto-routes to Europe PMC when strict unsupported filters are present.

### AC7: search-all article section upgraded
- `search_all.rs` article dispatch calls source-aware `search_page`.
- Article section uses relevance sort to prioritize entity-ranked results.

### AC8: Output and docs updated
- Markdown article search output is source-aware (grouped by source).
- CLI list/help docs include new `--source` option.

## Edge Cases

- Autocomplete no-match/error: fallback to raw token in PubTator query.
- PubTator returns rows without PMID: drop row (dedup key is required).
- `--source pubtator` with only unsupported filters and no queryable tokens:
  return `InvalidArgument`.
- `sort=citations` under PubTator: fallback to relevance (documented behavior).
- Federated dedup can reduce returned rows below `limit`; this is expected.

## Testing Strategy

- Unit (wiremock) for `src/sources/pubtator.rs`:
  - endpoint path/query params
  - API-key forwarding
  - deserialization and API error surfacing
- Unit (pure) for `src/transform/article.rs`:
  - PubTator -> `ArticleSearchResult` mapping
- Unit (pure) for `src/entities/article.rs`:
  - backend planner rules
  - PubTator query build/normalization fallback
  - dedup ordering behavior
- Integration (mocked upstreams via env base overrides):
  - `search_page(..., All)` concurrency + graceful degradation
  - single-source pagination with offset
- CLI parse tests:
  - `search article --source pubtator`
  - invalid source value rejection

## Notes

- `search()` helper callsites in cross-entity commands (`variant/disease/gene/pathway`)
  currently use the convenience wrapper, not `search_page()`. Keep wrapper behavior
  explicit during implementation to avoid accidental behavior drift.
- No `spec/` directory exists in this repo at this step, so spec-writing checks are
  not applicable for this ticket.
