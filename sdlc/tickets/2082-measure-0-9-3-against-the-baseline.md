---
priority: 4
---

# 2082 — Measure both releases under comparable conditions

Status: OPEN.

Milestone: 0.9.3
Depends on: 2073
Depends on: 2081
Depends on: 2094
Depends on: 2122

Depends on: 2088
Depends on: 2095
Depends on: 2120
Depends on: 2128
Depends on: 2129
Depends on: 2139
Depends on: 2149
Depends on: 2151
Depends on: 2152
Depends on: 2158
Depends on: 2159
Depends on: 2160
Depends on: 2161
Depends on: 2162
Depends on: 2164
Depends on: 2165

Depends on: 2166

Depends on: 2167

Depends on: 2168

Depends on: 2169
Depends on: 2172
Depends on: 2173
Depends on: 2174
Depends on: 2177

## Outcome

Fresh v0.9.2 and release-candidate measurements run on the same build machine with the same corrected harness, toolchain, workloads, provider settings and cache preparation, and the baseline record reports raw samples, honest comparisons and remaining failures.

- Starts from: `tools/measure-0-9-3-baseline` and `sdlc/records/2069-0-9-3-baseline.md`. Earlier cross-host, timed-out and failed-suite values are historical and superseded, not a valid before column.
- Keeps: exact immutable product revisions, the exact corrected harness revision, tool versions, command arguments, environment variable names without secret values, suite exit results and raw samples. Do not modify either measured product revision to make the harness run; distinguish harness compatibility work from product changes.
- Changes: claims `tools/measure-0-9-3-baseline` `sdlc/records/2069-0-9-3-baseline.md` `sdlc/release-checklist.md` `docs/reference/release-process.md` `CHANGELOG.md`. Run both products anew through the build job under matching settings. Alternate warm and cache-off pairs, prefetch consistently and isolate each cache-off sample so it cannot destroy the warm cache. Compute a true median, including the even-sized case.
- Proof: report routine lint/test/spec timings, build and release measurements, product test-code share, and comparable coverage using 2120's corrected collection; a failed or missing collection is a separate completeness result. Record 2117's clean and forced-failure behavior. Include five live search-all samples with GWAS errors, ten-run drug-card comparisons with output checksums, and day-cached GWAS/openFDA comparisons. Separate local product cost from live-provider timing and timeouts. Mark each applicable approved target met or missed with its measured number; retain the local `make test` and `make spec` targets and the approved test-code-share target, replace obsolete CI timing with local measurements, and report thirty-second slow-test warnings without failing solely on time. Warm-cache wins are observations, never an acceptance precondition. Explain any unavailable row rather than inventing a value.
- Release exit: after this ticket and every other release ticket land, run the entire manual untagged, nonpublishing release workflow on final main using 2161's rehearsal mode. Record the actual run and immutable revision in pm and report the result in chat; do not create a follow-up commit solely to record that final run. Repair any failure before repeating the final rehearsal. Ian alone creates the release tag.
- Defers: publication to Ian; no first-attempt success requirement and no automatic branch CI requirement.

PM decision: 2026-10-17. Measure both versions on one build machine. This ticket lands last; the final rehearsal follows all landings, avoiding a self-referential requirement to commit proof for its own final commit.

PM decision: 2026-10-17. Coverage uses the existing groups. The test deletion, parallel lanes and audit move land before this measurement.

PM decision: recurrence tickets 2175 and 2176 remain open through the final release rehearsal. Non-recurrence across the remaining full checks and rehearsal is sufficient for closure, with clean-check counts recorded; 2176 must include checks after the cache changes. They therefore do not block final measurement. All remaining product bugs, including 2172, 2173 and 2174, still precede this measurement. If a recurrence occurs, its fix precedes release.

Coordinator outcome before measurement: Replace the unusable cross-host before/after column with fresh, comparable measurements of immutable v0.9.2 and the repaired0.9.3 product atda3be5638db5f6b21b395998e412e6bbe9587b0c on the same build host. Every product-bug prerequisite is now complete;2174's unchanged MCP case passed fifty repetitions and its reviewed finding landed. Only2175/2176 observation remains open by the PM's explicit exception through final rehearsal. Existing code supplies the corrected measurement and coverage harness; prefer it unchanged. No product fix is planned. Before any necessary harness edit, report the actual command failure, exact correction and claimed files. Preserve both measured product revisions and record the separate immutable harness revision. Final measurements must wait for the current mandatory full check of main to finish so this repository does not load the host while timing itself. Run comparisons through build jobs, retain raw samples and honest completeness/target results, and separate provider variability from local cost. No new optimization or passing-warm-cache requirement is authorized by a missed target. Reuse qualified2120 coverage tooling and settings; do not run an extra coverage preflight before the final collections. The final untagged nonpublishing rehearsal follows this ticket's landing, not this record.
