# Output-footprint replay corpus is nondeterministic under load

Severity: should-fix. It sits inside the `test` gate rung, so it can fail any
flight for reasons the ticket did not cause.

`tests/test_output_footprint_benchmark.py::test_offline_corpus_is_
deterministic_and_reports_real_token_counts` calls `collect()` twice and
asserts equality. Under CPU contention the two runs disagree -- 2,851 versus
2,924 bytes for the full article search was one observed pair.

It is load-dependent, not simply broken: 13 consecutive passes on an idle
machine, and a reproducible failure inside a flight running alongside three
other builds. First seen by the agent flying ticket 0875; that agent filed it
in its worktree and the report was nearly lost when the run was discarded.

Likely shape: concurrent fanout merged in completion order. Either pin the
order or drop the byte-exact assertion and assert what is actually invariant.
This will get worse now that channels genuinely fly in parallel.

## Exact failure, captured 2026-08-08 23:50 in ticket 0875's flight

Only `article_search_compact` drifts. Everything else is byte-identical
between the two `collect()` calls:

    article_search_compact   output_bytes 1434 vs 1439
                             token_estimate 403 vs 405

`article_search_full`, `variant_search`, `gene_get_sections` and
`trial_search` all match exactly, and the ratchet passes in both. So the
instability is in the compact article-search surface alone, not the harness
or the tokenizer -- a small unpinned field or ordering in that one path.

**This blocks every biomcp flight, not just 0875.** The test sits in the
`test` gate rung, so it is a coin flip in front of the whole backlog.

Recommended fix, for review: compare everything except the two drifting
numbers and keep asserting the ratchet ceilings. The test's job is guarding
output size; byte-exact equality is stricter than that job needs. Fixing the
underlying field is better if it is cheap to find.
