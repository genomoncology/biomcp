---
flow: build
priority: 4
deps: []
---

# 1201: Discover runs its suggested article search inline

## Goal

`biomcp discover <query> --search` executes the article keyword search it
already recommends when no concepts resolve, and returns that search's results
inside the same response as the concept metadata — one call instead of two.

## Current Facts

When `discover` resolves no concepts it prints
`No biomedical entities resolved. Try: biomcp search article -k "<query>" --type review --limit 5`
(`${empty_results_article_fallback_note}`, `src/entities/discover.rs:2473`) and
emits the same command in `_meta.next_commands` and `_meta.suggestions`
(`to_discover_json`, `src/render/json.rs:282`). The caller must run that
command separately. Observed 2026-09-16: `biomcp discover "vemurafenib
resistance"` resolved no concepts and printed the suggestion with no results
inline. The command string is already built by
`review_article_fallback_command` (`src/entities/discover.rs:2455`) with the
existing `NextCommand` quoting, so the flag runs exactly what the note
recommends. `discover` is a raw-MCP-only surface; no typed tool changes.

## Design

### Flag

`DiscoverArgs` (`src/cli/system/mod.rs:189`) gains:

```rust
/// Run the suggested article keyword search when no concepts resolve
#[arg(long)]
pub search: bool,
```

Help text is exactly that doc comment. The flag is opt-in and inert when
concepts resolve: with `--search` and one or more concepts, the response and
every request are byte-identical to today's run.

### Inline search

In `src/cli/discover.rs::run_outcome`, after `resolve_query_with_options`
returns zero concepts, and only when `args.search` is set:

- Build the same article search the suggested command describes:
  `ArticleSearchFilters` with `keyword = Some(<trimmed query>)`,
  `article_type = Some("review")`, `limit = 5`, all other fields at their
  defaults, executed through the same entity entry point the CLI
  `search article` command uses (`crate::entities::article::search(&filters, 5)`
  returns the `Vec<ArticleSearchResult>` that the search JSON serializes).
- The executed request must be exactly what the suggested command would
  produce: request-log tests compare the fixture traffic of
  `discover --search` against `search article -k <query> --type review --limit 5`
  and require identical provider request sets.
- An error from the inline search propagates as the command error; the
  fallback is the point of the flag, so a silent downgrade to the note-only
  response would hide it.

### Frozen response additions

- JSON gains exactly one optional member, present exactly when the inline
  search ran and completed:

  ```json
  "article_search": {
    "command": "biomcp search article -k \"<trimmed query>\" --type review --limit 5",
    "returned": 3,
    "results": [ ...article search result objects... ]
  }
  ```

  `command` is the exact `review_article_fallback_command(query)` output
  (already quoted by `NextCommand`). `results` are the same
  `ArticleSearchResult` objects the search JSON serializes, in the same
  order; a test pins that the array is byte-identical to the `results` array
  of a direct `search article -k <query> --type review --limit 5 --json` run
  on the same fixture.
- All existing members (`concepts`, `notes`, `_meta.next_commands`,
  `_meta.suggestions`, and the rest) keep today's values; the note and the
  suggested command remain, because they stay true.
- Markdown gains one section after the existing note:

  ```text
  ## Article search

  <the body `search article` renders for the same filters and results,
  identical byte-for-byte except the header line>
  ```

  The body — the `Found N articles` line, the source-status lines, and the
  result table — comes from the same renderer call the search command makes
  (`article_search_markdown_with_footer_and_context`), with the same
  pagination footer the direct search builds for offset 0 and limit 5, and
  the header line rendered as `## Article search` instead of
  `# Articles: <query>`. The header becomes a parameter of that shared
  renderer (default `# Articles: <query>`); a test renders the same fixture
  results through both call sites and pins byte-equality apart from the
  header line. The `--search` section is absent when concepts resolve.

## Fixtures

The discover spec family is the disease-survival fixture
(`spec/fixtures/setup-disease-survival-spec-fixture.sh`), which already serves
OLS4 with a no-match playbook query and writes
`$BIOMCP_DISEASE_SURVIVAL_REQUEST_LOG`. Extend it with an article search route
(`/graph/v1/paper/search` under a Semantic Scholar base the fixture adds and
exports as `BIOMCP_S2_BASE`, which the fixture does not serve today) that
returns a fixed three-row payload for the fallback query, plus the matching
request-log lines in the existing log format.

Focused tests (Rust, fixture-backed, in the discover test module):

1. `--search` with zero concepts returns `article_search.returned == 3` and
   `concepts == []`, and the `article_search.results` array is byte-identical
   to a direct search call's `results` array on the same fixture.
2. Without `--search`, zero concepts produce today's response byte-for-byte
   (no `article_search` member, no search request in the log).
3. `--search` with resolved concepts is byte-identical to today's response and
   makes no search request.
4. Request parity: the inline run's provider request set equals the direct
   `search article -k <query> --type review --limit 5` run's request set.
5. Inline search failure propagates as the command error.
6. Markdown: the pinned `## Article search` section renders byte-exactly and
  agrees with the direct search render minus the substituted header line.

Executable spec (`spec/surface/discover.md`, the "No-Match Discover Queries
Fall Back to Article Search" section): one JSON block
(`discover "<fixture query>" --search --json` → `concepts == []`,
`article_search.returned == 3`, and the first result's identifier) and one
Markdown block pinning the `## Article search` heading and a row line.

## Acceptance

The flag exists with the exact help text; the six focused tests and both spec
blocks pass; `make lint`, `make test`, and `make spec` pass on the gate host at
the pushed SHA; the package path count is unchanged. `discover` without
`--search` is byte-identical on JSON and Markdown, and `discover --search`
with resolved concepts is byte-identical to today.

## Dependencies

None.

## Boundaries

One inline search only (no retries beyond the search's own bounded behavior, no
pagination chaining, no other fallback command). No change to concept
resolution, OLS4 routing, the note text, or the suggested-command string. No
new typed MCP tool and no catalog change; raw MCP execution reaches the flag
through the existing raw-tool path without extra work. No change to
`search article` itself. No caching of the inline results beyond whatever the
existing HTTP cache already does.

## Complexity

- Contract score: 1 (one new flag with several explicit cases: zero
  concepts, resolved concepts, failure, both output modes)
- State and timing score: 0 (one additional bounded request; no persistent
  state)
- Reach score: 1 (discover entity, CLI dispatch, JSON and Markdown renderers)
- Proof score: 1 (byte-agreement with the direct search, request parity,
  mode-specific goldens)
- Cost of error score: 0 (a wrong request is a cheap local correction; no
  lasting effect)
- Total: 3
- Minimum level floor: none
- Final level: 2
- Reasons: one conditional dispatch with byte-agreement proofs; no state or
  compatibility decision
- Selected model: gpt-5.6-luna, high reasoning (level 2 implementer)

## Review

- Design review: pending
- Code review: pending
