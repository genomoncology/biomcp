---
priority: 1
---

# 2277 — Keep disease pages in replay storage

Status: OPEN.

Milestone: 0.9.4

## Outcome

The two recorded melanoma query pages live once in canonical replay storage, and every fixture consumer and manifest points to that same captured data. The live checker can inspect them without relaxing its path boundary.

Changes: `testdata/disease-resolution/melanoma-page-one.json`, `testdata/disease-resolution/melanoma-page-two.json`, `testdata/replay/responses/mydisease/*`, `testdata/replay/manifest.json`, `src/entities/disease/resolution/paging_tests.rs`, `tests/test_article_disease_filter.py`, `CHANGELOG.md`.

## Evidence

Final 2258 collection found two manifest query routes outside the checker replay root: ../disease-resolution/melanoma-page-one.json (from=0, size=50), SHA256 d536ba05416270d607c4ded5bb810aaba7e4c6d479c9cb647cee889c2ea0d1ed; page two (from=50), SHA256 18e6655ad6bfa09bf3f5100fb4190502210f3fce07a36a37cf6d56569abff06a. Existing melanoma-detail is already canonical. Actual source inventory and replies are retained in private M5 experiment 2096-biomcp-2258-live-collection, job 2258/1792447971557-46385. Current stable main is 1c4753a453569d17dd66a13eb3d312ab8a5af4d4.

## Keeps

Move the existing public captures without changing a byte; preserve their request and provenance. Update all readers found by search. Do not duplicate payloads, invent source facts or weaken the checker path guard. No runtime behavior or new provider calls. Manifest work is only the two fixture path fields; the observed-header lane owns metadata and 2278 owns other response captures. Keep those independent edits intact on integration.

## Proof

Use existing checker to show the two path refusals on current main, then demonstrate canonical-path validation and all affected paging/article consumers. Verify both moved payload hashes and absence of stale readers. Run focused checks and necessary lint through pm job; no full gate. Report scope additions before widening beyond fixture readers.
