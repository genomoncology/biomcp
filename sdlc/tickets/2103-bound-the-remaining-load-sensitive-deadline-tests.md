---
flow: build
priority: 2
deps: [2102]
---

# 2103: Bound the remaining load-sensitive deadline tests

Status: COMPLETE.

Milestone: 0.9.3

Reviews: revision 1b1f848, accept

Reviews: revision 1b1f848, accept

## Evidence

Filed 2026-10-15 from lanes 2066 and 2067 under the PM's two-jobs doctrine (a timing test that fails only with concurrent jobs is a test defect). Each fails under two concurrent Linux check host jobs and passes otherwise; ticket 2102 already covers the rmcp contract deadline test, these are the rest observed the same day:

- add_clinvar_admits_rate_limiting_when_a_sustained_429_outlives_the_deadline (ClinVar 429 deadline bound).
- test_rich_cli_json_and_markdown_are_well_formed (60s subprocess deadline).
- test_rich_raw_mcp_matches_cli_byte_for_byte (60s subprocess deadline).
- test_rich_malformed_provider_page_is_an_mcp_error_not_an_empty_page (30s MCP select deadline).

## Work

Bound or shed the load sensitivity of each: raise the deadline for contract purposes, split the converging pair, or make the subprocess fixture deterministic. Each must pass with two jobs running.

## Acceptance

1. All four pass in a full suite while another Linux check host job runs concurrently, twice.

## Added 2026-10-15 (coordinator)

- New instance from 2071's lane: `tests/test_output_footprint_benchmark.py::test_offline_corpus_is_deterministic_and_reports_real_token_counts` - `article_search_compact` hit its 60s deadline only under lane-build contention on the Linux check host (load 8+); passes in CI at the same tip. Same class, this ticket's home.
- Also from 2100's lane: `human_mcp_command_dispatches_to_provider_once` and the full-suite timeout of `mcp_error_text_keeps_each_cli_refusal_line (corrected 2026-10-16: the earlier name was a lane-report shorthand; this is the real test)` - load-only failures, pass isolated with the export. Same class, this ticket's home.
- Also from 2073's lane (evening runs): `raw_mcp_rejects_contradictory_variant_filters_and_non_trial_batch_source` and `test_rich_hostile_paper_id_is_contained_and_never_a_command` join the class - contention-only, isolated runs pass.
- Also from 2105's lane: `test_disease_survival_setup_reaps_ppid_one_marker_orphan` (same module, PPID-transition wait, tripped once under two concurrent module runs) - same family, this home unless 2080 claims it first.
- Also from 2102's lane: `entities::drug::get::tests::drug_card_sources_run_at_once_and_keep_the_undelayed_output` (2090's parallelism assertion; four 1s-held replies answered in 2.89s at load 4-6) - load-fragile timing assertion, this home.


## Added 2026-10-15 (developer, coordinator-directed): the shared mechanism

Working the fixes found one mechanism behind this whole family of "fails under two gate jobs, passes in CI" tests: every spawned `biomcp` process that does not pin `BIOMCP_CACHE_DIR` inherits the default user cache root, and shared-client construction runs startup maintenance (secure, migrate, epoch, lock, startup clean) under a lock on that root. On a gate host with another job running, the subprocess waits on every other biomcp process's maintenance and on a cache that grows with workstation use, so its wall time balloons until whichever deadline fires first; CI hosts start with an empty user cache, which is why the same tests stay green there. Observed directly: the output-footprint corpus outran a 240 s pinned article-search deadline under a concurrent job while inheriting the default root, and passed in 8.4 s once `collect` gave both passes one private empty root. The fixes in this ticket therefore give every spawned binary a fresh empty cache root (the two rmcp contract spawns, every author-surface CLI and serve child, the corpus runner) and keep the deadline proofs bounded and load-scaled rather than removing them. This explains the load sensitivity of every test named above; the ClinVar watchdogs were the same class in-process (client construction inside the measured window).
- New instance shape from 2081 batch 3 (2026-10-16): the same output-footprint test drifted by ONE BYTE (article_search_full 2524 vs 2525) under load instead of hitting the deadline - load-induced nondeterminism in the footprint corpus, same home.

