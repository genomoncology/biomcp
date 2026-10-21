# 2336 — Recover from transient literature source failures

Status: OPEN.

Milestone: 0.9.4

## Outcome

A literature search survives a transient Europe PMC, Semantic Scholar or PubTator3 failure where it can. After an HTTP 503, 502, 429 or timeout, BioMCP retries once with backoff, honoring Retry-After within the command's deadline. If the retry also fails, it returns every result the other routes found and names the failed route, its error class and that a retry ran. Dated, newer-paper searches get the same handling. No retry follows a 4xx other than 429, and provider pacing rules stay in force.

- Proof: outside-in replay tests with a Europe PMC 503 then success return complete results; 503 twice returns partial results naming Europe PMC with retry_attempted; a timeout on a dated search behaves the same; a 400 is not retried; the total time stays within the command deadline.

## Evidence

- On 2026-10-21, research ran five fresh agents with BioMCP 0.9.4 (commit cdb9f4a1b) on literature searches for five hereditary cancer variants, with no answer key. Exact commands and output are in research's protected store under runs/0010/five-v1.
- Item 3: Europe PMC returned HTTP 503 or timed out in four of five cases, including every dated search. Semantic Scholar and PubTator3 timed out once each. Each search came back incomplete.
- src/sources/europepmc.rs already attaches retry source context (lines 87-173). Start by recording what it retries today and why these four cases still ended incomplete.
- Ian asked on 2026-10-21 to clear every known issue in 0.9.4.

## Changes

`src/error.rs` (preserve classified provider warnings), `src/sources/mod.rs`, `src/sources/literature_retry.rs`, `src/sources/semantic_scholar.rs`, `src/entities/article/search.rs` (source-leg failure handling only), `src/entities/article/backends.rs` (partial-page and Semantic Scholar error reporting only), `src/entities/article/variant_search.rs` (error event and source-status reporting only), `tests/test_literature_transient_retries.py`, `tests/feature-links.json`, `tools/check-source-registry.py` (classify shared retry helper), `docs/user-guide/variant.md`, `CHANGELOG.md`.
