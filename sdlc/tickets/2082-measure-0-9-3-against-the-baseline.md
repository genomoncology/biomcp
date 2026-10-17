---
priority: 4
---

# 2082 — Measure both releases under comparable conditions

Status: OPEN.

Milestone: 0.9.3

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

## Outcome

Fresh v0.9.2 and release-candidate measurements run on the same build machine with the same corrected harness, toolchain, workloads, provider settings and cache preparation, and the baseline record reports raw samples, honest comparisons and remaining failures.

- Starts from: `tools/measure-0-9-3-baseline` and `sdlc/records/2069-0-9-3-baseline.md`. Earlier cross-host, timed-out and failed-suite values are historical and superseded, not a valid before column.
- Keeps: exact immutable product revisions, the exact corrected harness revision, tool versions, command arguments, environment variable names without secret values, suite exit results and raw samples. Do not modify either measured product revision to make the harness run; distinguish harness compatibility work from product changes.
- Changes: claims `tools/measure-0-9-3-baseline` `sdlc/records/2069-0-9-3-baseline.md` `sdlc/release-checklist.md` `docs/reference/release-process.md` `CHANGELOG.md`. Run both products anew through the build job under matching settings. Alternate warm and cache-off pairs, prefetch consistently and isolate each cache-off sample so it cannot destroy the warm cache. Compute a true median, including the even-sized case.
- Proof: report routine lint/test/spec timings, build and release measurements, product test-code share, and comparable coverage using 2120's corrected collection; a failed or missing collection is a separate completeness result. Record 2117's clean and forced-failure behavior. Include five live search-all samples with GWAS errors, ten-run drug-card comparisons with output checksums, and day-cached GWAS/openFDA comparisons. Separate local product cost from live-provider timing and timeouts. Mark each applicable approved target met or missed with its measured number; retain the local `make test` and `make spec` targets and the approved test-code-share target, replace obsolete CI timing with local measurements, and report thirty-second slow-test warnings without failing solely on time. Warm-cache wins are observations, never an acceptance precondition. Explain any unavailable row rather than inventing a value.
- Release exit: after this ticket and every other release ticket land, run the entire manual untagged, nonpublishing release workflow on final main using 2161's rehearsal mode. Record the actual run and immutable revision in pm and report the result in chat; do not create a follow-up commit solely to record that final run. Repair any failure before repeating the final rehearsal. Ian alone creates the release tag.
- Defers: publication to Ian; no first-attempt success requirement and no automatic branch CI requirement.

PM decision: 2026-10-17. Measure both versions on one build machine. This ticket lands last; the final rehearsal follows all landings, avoiding a self-referential requirement to commit proof for its own final commit.

PM decision: 2026-10-17. Tests, tooling and dependency work moved to 0.9.4, so this ticket no longer waits on them, and coverage uses the existing groups. Report the test-code share as measured.
