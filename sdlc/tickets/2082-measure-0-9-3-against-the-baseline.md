# 2082 — Measure 0.9.3 against the baseline

Status: OPEN.

Milestone: 0.9.3

## Outcome

Every number in the 2069 baseline is measured again on the 0.9.3 release candidate, side by side, and the 0.9.3 tag passes its first rehearsal.

## Evidence

- Ian asked for before-and-after numbers for 0.9.3; 2069 holds the before.

- Starts from: the 2069 script and record.
- Keeps: the same host, conditions and commands.
- Changes: claims `tools/measure-0-9-3-baseline`. Run the script on the release candidate; add the after column to the 2069 record with each change named; add a targets table for the hard targets in [the 2026-10-14 decision](../decisions/2026-10-14-delete-tests-boldly-and-split-the-suites.md), in the numbers the tickets enforce: median ticket-branch CI wall time 11 minutes or less (2094), `make test` 200 s or less (2094), `make spec` 200 s or less (2080), no routine test over 10 s (2095), and the test-code share against the target set after 2081's first batch.
  - The CI block reads green `ci.yml` runs whose `head_branch` starts with `ticket/`, so `main` runs that still carry the heavy jobs stay out. A run counts as full when its `rust-tests` job ran and succeeded, not when it used more than 30 runner-minutes, since a ticket-branch run sheds about 42 runner-minutes once 2075 moves the heavy jobs to `main`. It prints the median and range over the last ten full runs.
- Proof: the record with both columns and each target marked met or missed with its number; the 0.9.3 rehearsal run passes on its first try.
- Defers: nothing.

## Amendment, 2026-10-14

Fix the run-time measuring method.

- A run-time investigation on the build host found the 2069 run-time block skews its result. It runs all five warm runs before all five cache-off runs, so provider drift lands on one mode; cache-off runs share the warm cache folder, and a cache-off request deletes the stored entry it touches (http-cache's no-store path), so any later warm run on that folder goes back to the network; and the script inherits whatever provider keys the host sets, which changes what can be cached. The GWAS section's 12-second cut also made three warm `search all` runs read exactly 12.02 s: those numbers are a timeout, not a speed.
- Changes: claims `tools/measure-0-9-3-baseline`.
  - The run-time block alternates warm and cache-off runs, one pair at a time.
  - Each cache-off run gets its own copy of the prefetched cache folder.
  - The block prints the names, never the values, of the provider key variables set in its environment, and the GWAS section's error field for each `search all` run.
- Proof: re-measure the run-time block on v0.9.2 with the fixed method and on the release candidate, so before and after compare like with like; the record shows both, and warm comes out at or below cache-off for every command whose providers allow caching.

## Amendment, 2026-10-16

The 0.9.3 adversarial review found the before column unsound for some rows and several speed claims never measured.

- Measure v0.9.2 again, with the fixed method, for the coverage rows (2069's outside-in and combined numbers came from runs with 1 failed integration test and 26 failed Python tests) and the run-time rows (three warm `search all` runs read 12.02 s, the GWAS section's timeout). Mark the old 2069 numbers superseded in the record.
- Compute CI medians as true medians; the script's `list[n//2]` (lines 780 and 787) takes the upper median.
- Record the live proofs 2089, 2090 and 2091 left undone: five live `search all` runs with the median and GWAS errors (2089); ten-run main against v0.9.2 medians for `get drug` with output checksums (2090); before and after medians for the day-cached GWAS and openFDA calls (2091).
- Record 2117's forced-fail and clean coverage outputs.
- Order: after 2120 and 2123.
