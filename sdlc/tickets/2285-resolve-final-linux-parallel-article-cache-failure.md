---
priority: 1
---

# 2285 — Resolve final Linux parallel article cache failure

Status: OPEN.

Milestone: 0.9.4

Reviews: revision 3e6575b, accept

## Outcome

Diagnose the final Linux parallel article-source cache failure and prove that the supported cold-admission and warm-reuse behavior works. Correct answers, isolation, meaningful concurrency and genuine admitted-response reuse remain required. Decide from observed failure evidence whether the cause is runtime behavior or the test contract.

Changes: `tests/test_http_cache_fail_soft.py`, `CHANGELOG.md`, `sdlc/records/2285-parallel-article-cache-qualification.md`.

## Evidence

Fresh final Linux job 2233/1792455276832-365318 on actual main 4c582db4186837893e4839f6d47e5e60a129aae7, tree ab573395596a0b3530b76a7d3edba4d97f6686c7, passed 4,279 routine Rust cases with 34 declared skips, the explicit long selection and Rust inventory. During its existing sequential full Python invocation, test_http_cache_fail_soft.py::test_parallel_article_sources_fill_cold_cache_then_reuse_answers[command1] failed at about 39 percent; command0 passed. The invocation continues without retries, so the traceback is pending. No docs or full-feature stage follows a failing Python invocation.

New raw evidence remains separately under experiments/170-biomcp-2233-linux-offline-evidence/final-1792455276832-365318. The experimenter preserved accessible replay request logs, environment information and case directory immediately. The case cache belonged to replay_env borrow()'s TemporaryDirectory and was already removed by fixture teardown when failure appeared; pytest basetemp alone cannot retain that database. Preserve this limit and do not invent the missing state.

Ticket 2275 previously corrected this parallel cache proof under the supported best-effort admission policy. Ticket 2283 separately corrected the initialized MCP/CLI proof after controlled evidence. Those precedents are context, not the cause of this new failure. The actual-main full check passed on the build host before this final Linux attempt.

## Brief

Read the complete final traceback and relevant retained evidence before choosing a fix. While the full invocation finishes, read the existing proof and relevant prior records and prepare a bounded diagnosis; do not start a second build job yet. Restate the outcome and reproduce the named failing contract on current main using existing local fixtures and controls. Preserve exact returned-data assertions, isolation and strict warm reuse without source requests. Never loosen a count, suppress a failure, add arbitrary retries or change cache policy to make the test pass.

The initial claim permits only the named test and owned record/changelog. If evidence identifies a runtime defect or requires a shared helper change, report the cause and exact additional paths before editing outside it. A successfully acknowledged write disappearing requires a runtime fix. Explicitly skipped bounded admission remains supported. No live providers, new harness, keys, threshold changes or clinical judgment.

## Proof

Retain the original failure and the unavailable-case-database limitation. Use one bounded focused diagnosis through pm job after the coordinator releases a slot, with expected-red and corrected-green proof and meaningful sensitivity to the identified defect. Preserve any newly reproduced failed case database before teardown using the smallest existing control. Run affected tests and required lint; reuse unchanged artifacts. Push the final green implementation once, report its exact revision and evidence, and stop for fresh review. No full gate, landing or tag by the developer.

Keep: experiments/170-biomcp-2233-linux-offline-evidence — retain all initial, failed and renewed final Linux attempts, including the new parallel-article failure.

## Progress

- 2026-10-19 started

## Coordinator verification

The coordinator read the complete test diff and safe record, checked the clean final branch and its test hash against complete-inputs.json, and read the final 29-case pass and required-check results. Final focused job 2285/1792458135864-11811 exited zero. The original Linux failure remains failed and its missing database limit remains explicit. Only this repair is ready; landing it alone unlocks final qualification. A fresh reviewer must inspect the final revision and evidence before landing.

Keep: experiments/2285-biomcp-parallel-article-cache — retain all controlled attempts, the intermediate fixture failure, final affected results, source hashes and owned binary. OWNER identifies biomcp/2285.

## Final Linux reuse ruling

The PM approved reusing the unchanged Rust routine, long-selection and inventory proof from 4c582db4186837893e4839f6d47e5e60a129aae7 only when the exact repaired-tip diff contains Python tests and change records, with no Rust, Cargo, build, script or fixture-file changes. Otherwise rerun the whole Linux suite. A fresh clone, exact rebuilt CLI, full corrected Python suite, strict docs and shipped-feature checks under network isolation remain required, as do the full actual-main build-host check and release rehearsal. Keep the failed attempt marked failed.
