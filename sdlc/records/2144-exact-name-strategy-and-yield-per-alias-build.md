# 2144 build record: exact-name strategy and yield per alias

Branch `ticket/2144-search-one-exact-variant-name-and-report-yield-per-alias`,
rebased onto main `159c68610` (after the 2131 LitVar2 landing and its alias-arm
repair, and the 2142 landing; both 2144 dependencies — 2130 and 2138 — landed
before the original branch point).

## Gap recorded before any code

The reproduction lives in the ticket. Run on the macOS gate host at main
`f9a11e81e`, pm job `1792211931779-13479`, live providers, `--no-cache`:

- `biomcp --json variant articles "BRAF V600E" --strategy exact --limit 5
  --no-cache` exits 2 with `invalid value 'exact' for '--strategy
  <STRATEGY>' [possible values: union, annotation, lexical]`.
- `biomcp --json variant articles "BRAF V600E" --strategy lexical --limit 5
  --debug-plan --no-cache` exits 0; the debug plan's top-level keys carry no
  per-alias per-source yield (`any per-alias yield key anywhere: False`) while
  several aliases ran.

## What changed

- `variant articles --strategy exact NAME` searches the caller's string with
  no resolution and no alias expansion: one `exact_name` route with one
  phrase-quoted query per source — PubMed
  `"NAME"[Title/Abstract]`, Europe PMC `TITLE_ABS:"NAME"`, Semantic Scholar
  through the bulk phrase endpoint, PubTator as search text. Resolution and
  canonical equivalence are marked inapplicable and report plain-words skipped
  rows; `provider_validation` gains a `not_attempted` status so the response
  never claims MyVariant missed the variant. The ClinVar VariationID refusal
  and the variant-input parser no longer gate the exact strategy; any
  non-empty name is searchable.
- `--debug-plan` gains `alias_yields`: one row per route, alias, and source
  with `hits_received`, `hits_kept` (rows still carrying that alias's
  provenance fact after the merged pool settles), and `cut_by_cap` (the
  25-result fetch cap cut the source's list, a truncated page, or a provider
  total beyond the delivered rows). Rows are recorded for the strict,
  exact_lexical, best_effort_free_text, and exact_name routes and serialize in
  deterministic route/alias/source order; the field is omitted when empty.
- The MCP `variant_articles` tool accepts `strategy: "exact"` beside union,
  annotation, and lexical; the CLI list surfaces and docs name all four.
- CHANGELOG carries the 0.9.3 New features bullet citing (2144), beside the
  landed 2131 bullet.

## Tests

Recorded-reply tests in `src/entities/article/variant_search.rs`:

- `exact_mode_sends_one_phrase_quoted_query_per_source_without_resolution` —
  asserts each provider's exact phrase query from the recorded request log
  (decoded from the wire), no MyVariant query, no PubTator autocomplete, no
  ClinGen CAR call, no LitVar2 request, the unresolved/not_attempted
  resolution, the absent canonical equivalence, the four reached `exact_name`
  sources, and the exact name's pubmed yield row.
- `debug_plan_reports_yield_per_alias_per_source_with_cut_lists` — a resolved
  two-alias lexical search (MyVariant fixture hit carrying only the gene and
  `p.R1436Q`) shows each alias's per-source counts, a shared paper kept once
  with both aliases' yield attribution, and a cut list where Europe PMC's
  hitCount exceeds the delivered row.
- `zero_deadline_ledger_uses_independent_strategy_and_identity_oracles` covers
  the Exact strategy matrix: the empty-tail enrichment/identity rows report
  zero-work ok and the LDH rows stay inapplicable.

No new testdata fixtures were added (inline fixture replies only), so the
fixture boundary pin is unchanged.

## Gates at the post-rebase tip `791b9f5ae`

| Gate | Host | SHA | Job | Verdict |
| --- | --- | --- | --- | --- |
| Whole variant_search suite | m5 | 791b9f5ae | 1792227408877-24586 | 58/59 passed; fmt and lib clippy clean; the one failure is 2138's `no_entity_annotation_run_names_the_miss_instead_of_complete_empty`, which fails identically at main tip `159c68610` (job `1792227303520-20164`) |
| make lint | m5 | 791b9f5ae | 1792227512979-31574 | passed |
| make spec | m5 | 791b9f5ae | 1792227562813-39652 | 59 passed, 2 failed — both `spec/entity/clingen-erepo.md` |
| clingen-erepo page at main tip | m5 | 159c68610 | 1792227607658-50783 | fails the same two lines at current main, so that spec failure predates this branch |
| make test | yellow | 791b9f5ae | 1792227514674-3691894 | failed on `no_entity_annotation_run_names_the_miss_instead_of_complete_empty` alone — the inherited main-tip defect below; every other test that ran before the abort passed and the Python lane never started |

Earlier pre-rebase evidence: gap job `1792211931779-13479`; targeted new +
neighbor tests 5/5 at `1b61e5062` (job `1792214907410-52328`); variant_search
suite 56/56 at `05d6bd590` (job `1792221621682-73418`); make lint passed at
`05d6bd590` (job `1792221732163-88466`); make test passed at `05d6bd590`
(yellow job `1792221714906-3482572`, 1160 Python tests plus the Rust lane);
make spec at `05d6bd590` had the same two erepo lines failing (job
`1792221783631-97832`) and the same page failed at `f9a11e81e` (job
`1792221825021-99780`).

## Findings outside this branch

- Decision 2026-10-17 (coordinator): the LitVar2 test dependency is healed
  by 2140's branch, which pins `BIOMCP_LITVAR_BASE` in the no-entity test and
  lands first. This branch holds without its own pin to avoid colliding with
  that landing; if 2140's landing is withdrawn, the same one-line pin is
  authorized here.

- `entities::article::variant_search::tests::no_entity_annotation_run_names_
  the_miss_instead_of_complete_empty` fails at main tip `159c68610` (job
  `1792227303520-20164`): after the 2131 landing the annotation strategy also
  runs the LitVar2 route, and that test pins no `BIOMCP_LITVAR_BASE`, so its
  `results.is_empty()` assertion depends on the live LitVar2 service returning
  nothing for `ABCC8 R1436Q`. Nothing in this branch touches the LitVar2
  route or that test; the branch's own runs reproduce only that inherited
  failure.
- `spec/entity/clingen-erepo.md` fails identically at main tips `f9a11e81e`
  and `159c68610` (jobs `1792221825021-99780` and `1792227607658-50783`):
  the frozen ERepo fixture's bash and json checks exit 1. Nothing in this
  branch touches ClinGen eRepo.
- `cargo clippy --no-default-features --all-targets -- -D warnings` reports
  three findings at main tip `159c68610` (job `1792227637432-54711`):
  `unused import: std::fs` in `src/cache/manager.rs` and two
  borrowed-expression findings in `src/sources/tests/provider_network.rs`.
  The routine lint gate compiles without `--all-targets` and passes; these
  surface only under the all-targets shape.
