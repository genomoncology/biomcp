---
flow: build
priority: 10
---
# Report truthful search-all counts

`search all --counts-only` currently presents the fetched row count as a total
when a provider supplies no exact total. Changing `--limit` therefore changes a
supposed biomedical total. Adding `--debug-plan` also changes the article fetch
limit and can change the reported count. Debugging must never change a domain
result.

## Count contract

Each section carries:

- `returned`: rows fetched for this response;
- `total`: an exact total or null;
- `count_exact`: true only when `total` is exact; and
- `total_lower_bound`: null for an exact total, otherwise the number of rows
  proven to exist so far.

A provider-reported exact total or a successful, proven exhausted result set
may set an exact total. A fetch cap is never an exact total. A zero-row current
page is exact zero only when the provider reports exact zero or BioMCP proves
the entire result set is exhausted. If continuation remains possible, it is a
lower bound of zero with a null total. An unavailable or failed source has no
total.

Human output says `N`, `at least N`, or `unknown` as appropriate. Existing
source-outcome text remains visible. `--debug-plan` may add diagnostics but uses
the same fetch plan and leaves every count field byte-identical.

## Done when

- Local provider fixtures cover exact, lower-bound, exhausted, proven exact
  zero, a zero-row page with continuation, and failed sections.
- Changing `--limit` changes `returned` and perhaps the lower bound, never a
  guessed exact total.
- Adding `--debug-plan` changes only the debug projection.
- JSON and Markdown agree, including `--counts-only`.
- The temporary `current_counts_only_shape` exception is removed from
  `tests/test_cli_surface_contract_ratchet.py`.

This is a regression or unfinished edge of archived ticket 0206; do not alter
that historical record.

## Authorized test changes

Design commits may restate count and debug expectations in
`src/cli/search_all/tests/dispatch.rs`,
`src/cli/search_all/tests/format.rs`,
`tests/test_cli_surface_contract_ratchet.py`, and
`tests/test_public_search_all_docs_contract.py`. Existing routing, source
attribution, result rows, and follow-up links must remain covered.

The src line ceiling may rise by at most 180 lines.
