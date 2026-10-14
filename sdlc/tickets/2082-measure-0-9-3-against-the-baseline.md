# 2082 — Measure 0.9.3 against the baseline

Status: OPEN.

Milestone: 0.9.3

## Outcome

Every number in the 2069 baseline is measured again on the 0.9.3 release candidate, side by side, and the 0.9.3 tag passes its first rehearsal.

## Evidence

- Ian asked for before-and-after numbers for 0.9.3; 2069 holds the before.

- Starts from: the 2069 script and record.
- Keeps: the same host, conditions and commands.
- Changes: run the script on the release candidate; add the after column to the 2069 record with each change named.
- Proof: the record with both columns; the 0.9.3 rehearsal run passes on its first try.
- Defers: nothing.

## Amendment 2026-10-14: fix the run-time measuring method

- A run-time investigation on the build host found the 2069 run-time block skews its result. It runs all five warm runs before all five cache-off runs, so provider drift lands on one mode; cache-off runs share the warm cache folder, and a cache-off request deletes the stored entry it touches (http-cache's no-store path), so any later warm run on that folder goes back to the network; and the script inherits whatever provider keys the host sets, which changes what can be cached. The GWAS section's 12-second cut also made three warm `search all` runs read exactly 12.02 s: those numbers are a timeout, not a speed.
- Changes: claims `tools/measure-0-9-3-baseline`.
  - The run-time block alternates warm and cache-off runs, one pair at a time.
  - Each cache-off run gets its own copy of the prefetched cache folder.
  - The block prints the names, never the values, of the provider key variables set in its environment, and the GWAS section's error field for each `search all` run.
- Proof: re-measure the run-time block on v0.9.2 with the fixed method and on the release candidate, so before and after compare like with like; the record shows both, and warm comes out at or below cache-off for every command whose providers allow caching.
