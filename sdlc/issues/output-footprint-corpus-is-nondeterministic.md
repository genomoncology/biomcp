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
