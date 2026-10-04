# 1293 — bound article search time and report partial sources

Proposed 2026-10-03 by the BioMCP owner for the next 0.9 release. Status: accepted, ready to build.

## Evidence

- Experiment 432: tool calls took about 10,400 s against 1,500 s of model time. The slowest calls were `search article "SCN5A Brugada syndrome mutations" --limit 10` (206 s), `search article -k "tumor immune evasion mechanisms" --type review --full` (186 s) and a glioblastoma `--type review` search (180 s). At a 300 s agent budget, 21 of 60 agent cells timed out.
- Experiment 433: 10 of 120 TREC topics first returned zero rows, and 7 still failed with `source_unavailable` after three tries.

## Change

1. Measure which source and which step consume the time in those commands.
2. Give article search one overall deadline with a stated default. When it passes, return the rows already found and mark the slow sources as unavailable in the output.
3. Print the deadline and per-source timing in `--full` diagnostics.

## Retained behavior

Ranking, sources and output shape stay the same when every source answers in time.

## Proof

A spec case with a recorded slow source that returns partial rows and names the slow source. The three slow commands rerun with timing in the record.

## Deferred

Changing rate-limit handling for NCBI keys.

## Review

- Design review: ACCEPT 2026-10-03 on the first pass, dispatch 26de9b8f (fresh SWE-2 researcher, read-only).
