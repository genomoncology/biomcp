# BioASQ Ranking Calibration

This directory documents the repo-local calibration surface for article-ranking
tuning. It does not create a new benchmark lane; it records the worked-example
and fixture surface used to verify lexical, semantic, and weighted hybrid
ranking behavior inside Rust tests before future BioASQ sweeps.

## Verified automated scenarios

All verified automated cases live in the `#[cfg(test)]` module of
`src/entities/article.rs`.

| Scenario | Fixture surface | Rust test | Expected behavior |
|---|---|---|---|
| Worked example 1 | Five-paper semantic-vs-lexical fixture | `hybrid_default_weights_orders_example_one` | Default hybrid weights rank `C > A > B = E > D` |
| Worked example 2 | Same five-paper fixture | `lexical_mode_matches_current_ordering` | Lexical mode stays byte-identical to the current directness comparator |
| Worked example 3 | Entity-only no-semantic fixture | `hybrid_entity_only_falls_back_without_nan` | Hybrid degrades cleanly to lexical/citation/position scoring without NaN output |
| Worked example 4 | Same five-paper fixture with weight overrides | `hybrid_custom_weights_shift_ordering` | Lexical-heavy weights rank `C > B > D > A > E` |
| Zero-safe normalization | All-zero citations and all-zero positions | `hybrid_scoring_is_zero_safe` | Hybrid component normalization stays finite when citation or position maxima collapse |
| PubTator-only semantic clamp | Source-specific PubTator row with raw score `285.0` | `hybrid_uses_litsense2_signal_for_semantic_score` | PubTator-only rows contribute `semantic_score = 0.0` even when the raw score is large |
| Merged LitSense2 semantic preservation | PubTator-first duplicate plus LitSense2 duplicate | `hybrid_uses_litsense2_signal_for_semantic_score` | Hybrid ranking keeps the LitSense2-derived semantic signal even when the public merged `score` still comes from PubTator |
| Semantic-mode source gating | PubTator-only row vs LitSense2 row | `semantic_mode_ignores_non_litsense2_raw_scores` | Semantic ranking ignores non-LitSense2 raw scores and orders by the LitSense2-derived signal |

## Public bundle regeneration

Regenerate the recommended public historical bundle with the repo-standard
command:

```bash
uv run --quiet --script benchmarks/bioasq/ingest_public.py --bundle hf-public-pre2026
```

Use that bundle when you want to compare future ranking-weight sweeps against
the public BioASQ lane after the unit-test fixtures are green.

## Provenance pointers

- `benchmarks/bioasq/datasets/manifest.json`
- `benchmarks/bioasq/datasets/README.md`
- `docs/reference/bioasq-benchmark.md`

Those files remain authoritative for bundle ids, output layout, provenance
boundaries, and the public-lane versus official-lane runbook.

## Existing live JSON proof

The existing structural ranking-metadata proof remains in the active article
canary, and the keyword-default hybrid contract is exercised through
`spec/entity/article.md`, including zero semantic score on rows without
LitSense2 provenance. Live upstream result order still drifts, so the stable
calibration surface for ranking-order proofs stays in these Rust fixtures plus
the benchmark docs.

## Historical leads

These leads remain useful context, but they are not part of the mandatory
automated fixture set in ticket 149.

| Lead | Expected PMID(s) | Status |
|---|---|---|
| WDR5 / pancreatic cancer | `28741490` | Historical context only; not promoted to an automated fixture in this ticket |
| RUNX1T1 / m6A methylation | `32589708`, `25412662` | Historical context only; not promoted to an automated fixture in this ticket |
| Pds5b / Cornelia de Lange | unresolved | Historical lead; answer PMID is unresolved in current repo artifacts |
| etanercept / anti-TNF | unresolved | Historical lead; answer PMID is unresolved in current repo artifacts |
