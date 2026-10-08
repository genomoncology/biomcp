# 1293 — bound article search time and report partial sources

Proposed 2026-10-03 by the BioMCP owner for the next 0.9 release.

Status: COMPLETE.

Landed: 6bf7ca55.

## Outcome

Every article search finishes within its 60-second deadline and returns the rows that already answered, with each slow source named as degraded. A search where nothing answered still fails with `SourceUnavailable`.

## Evidence

- Starts from: Experiment 432: tool calls took about 10,400 s against 1,500 s of model time. The slowest calls were `search article "SCN5A Brugada syndrome mutations" --limit 10` (206 s), `search article -k "tumor immune evasion mechanisms" --type review --full` (186 s) and a glioblastoma `--type review` search (180 s). At a 300 s agent budget, 21 of 60 agent cells timed out. Experiment 433: 10 of 120 TREC topics first returned zero rows, and 7 still failed with `source_unavailable` after three tries.
- Keeps: Compact output unchanged. Five ordinary searches had identical keys and row order on both binaries. Section names and search row shapes stay the same.
- Changes: One `VariantArticleDeadline` per `search_page` invocation through the existing task-local provider seam, covering federated legs, single-backend plans, the Semantic Scholar enrichment batches, and the per-row metadata fallback loop. On expiry the page returns the rows that answered; sources that outlasted the deadline are reported `degraded` with a message naming the deadline. When both primaries fail, PubMed/Semantic Scholar/LitSense2 rows return as a partial page instead of `primary_error` discarding them. `--full` JSON carries `diagnostics.deadline_ms` plus per-source/per-stage `source_timings`.
- Proof: The three slow commands settle at the deadline with rows (1m0.037s, 1m0.072s, 1m0.034s, exit 0, 10 rows each). The recorded fixtures override the budget through `BIOMCP_TEST_ARTICLE_SEARCH_DEADLINE_MS`; at 8000 ms the search settles at 8.02 s with 50 partial rows and each held source named. Gates and CI in What landed.
- Defers: Changing rate-limit handling for NCBI keys.

## What landed

- Measurement: the three slow commands were rerun against this branch's binary and every one now settles at the deadline with rows instead of multi-minute stalls. `search article "SCN5A Brugada syndrome mutations" --limit 10` finished in 1m0.037s (ticket: 206s; a pre-change rerun on this worktree still blew a 240s external timeout after the PubTator3 leg timed out at 12s, so the missing time sat in the unbounded enrichment loops). `search article -k "tumor immune evasion mechanisms" --type review --full` finished in 1m0.072s (was 186s). `search article -k "glioblastoma treatment resistance mechanisms" --type review --full` finished in 1m0.034s (was ~180s). All three returned 10 rows, exit 0.
- Default: 60 seconds, matching the existing 60s `VariantArticleDeadline` budget that already bounds the same provider mix for variant article search. The federated legs keep their own 12s timeout, leaving ~48s for the sequential Semantic Scholar batches and per-row PubTator/Europe PMC metadata fallback that consumed the measured time.
- Gates: `make lint` exit 0. `make spec` exit 0 (the new held-source article spec case ran inside the article.md batch, 178 passed). `make test` exit 2 in this worktree's sandbox on pre-existing load flakes outside this change: `cache::manager::tests::stale_marker_tests::*` where `get` returns `None` immediately after `put` under parallel load (files identical to main; the three tests pass standalone, pass/fail/pass on direct rerun), and in one run `cli::stale_json_note_tests::*` plus two `entities::disease::get` stale-cache tests failing `Connection refused` to their own fixture servers. All 426 article-scope nextest tests passed, including the three new deadline tests; the same archive rerun minus `test(stale)` showed no other failures. On CI the same `make test` job passed end-to-end (canonical-gates test step, run 37155647495).
- CI: the first two pushes failed the canonical test gate on contract registrations this branch needed, not on the deadline behavior: the article CLI test-file layout list (diagnostics.rs), the UNPACED-ORIGIN fixture exporter allowlist, the env-var docs classification for the deadline test seam, and the packaged-file count (1379 -> 1385). The third push fixed all four; run 37155647495 finished green across all eight jobs (canonical lint/test/spec, repository and Windows contracts, generated sources, stress lane, full features, release panic).
- QA: KB ticket 0003, landed in the KB repository. The 7 TREC topics that failed in experiment 433 failed 32 of 35 runs on 0.9.1, each at about 12 seconds with `source_unavailable`; the branch returned rows on 35 of 35 runs in 12 to 19 seconds. No live run reached 60 seconds (0 of 116, slowest 18.6 s). Agent smoke timeouts at 300 s fell from 7 of 20 to 0 (leg B) and 4 of 20 to 0 (leg C), on a different day from the baseline. No defect found in scope.
- Code review: ACCEPT, dispatch 09bdfcbf (fresh reviewer, read-only), with four minor notes. The notes were not carried into this record before landing; the dispatch and the handoff message of 2026-10-04 hold them.

## Review

- Design review: ACCEPT 2026-10-03 on the first pass, dispatch 26de9b8f (fresh researcher, read-only).
