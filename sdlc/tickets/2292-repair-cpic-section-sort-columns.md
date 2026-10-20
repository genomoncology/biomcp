---
priority: 1
---

# 2292 — Repair CPIC section sort columns

Status: OPEN.

Milestone: 0.9.4

## Outcome

Current CYP2D6 frequency and guideline sections use supported CPIC sort columns and return provider data. Preserve deterministic pagination, existing output and source failure reporting. This is an API request repair; make no clinical judgment or recommendation changes.

Changes: `src/sources/cpic.rs`, `src/sources/cpic/tests/`, `tests/pgx_get_paging.rs`, `testdata/replay/`, `sdlc/records/2292-cpic-sort-columns.md`, `CHANGELOG.md`.

## Evidence

Approved capture job `2236/1792508969309-13393` sent the exact current keyless public production requests. The population_frequency_view endpoint returned HTTP400 because column allele does not exist. The guideline_summary_view endpoint returned HTTP400 because column guidelinename does not exist and suggested guideline_name. Nine other authorized captures returned HTTP200. No transient retries were used. Raw requests and replies remain in experiment2102 public-captures/2236.

## Scope

Restate outcome, exact reproduction and planned file claim before code. Inspect existing schema fixtures and upstream primary API documentation to identify supported ordering fields, including all copies of those request plans. Repair only this adapter class and its meaningful outside-in controls; no provider fallback or schema framework. Existing public keyless read-only current-route parameter refresh authorization applies; pace requests, keep metadata, retry only once after a transient error, and never call Semantic Scholar, KEGG or licensed sources. Do not repeat the known failed requests merely for records.

## Proof

The affected production request path fails with retained HTTP400 replies before the correction and uses verified valid deterministic ordering afterward. Exercise the actual CLI through exact recorded replies and preserve unexpected failure behavior. Run only affected tests on the build machine through pm job. A fresh reviewer accepts before landing.

Keep: experiments/2102-biomcp-2258-final-live-replay — preserve public-captures/2236 original failed CPIC replies and the bounded repair evidence while release admission consumes them.

## Pre-code restatement

Restore CYP2D6 frequency and guideline sections by replacing unsupported adapter ordering fields while preserving deterministic paging, output and failure reporting. The exact retained job is the reproduction; do not repeat its HTTP400 calls. Schema structs and receipted fixtures identify frequency name/population_group and guideline_name; verify full ordering before editing. The outside-in affected CLI section checks must fail on the old request plan and pass the repaired exact request, using the existing harness.

PM decision: fix this live user-facing bug for 0.9.4 at priority1. After repair recapture both routes under standing public keyless authorization, add them to the recorded set and existing live drift checker, and state in the changelog that both sections were broken against the current CPIC API.
