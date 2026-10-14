# 2069 — Record the 0.9.3 baseline

Status: OPEN.

Milestone: 0.9.3

Reviews: revision fda07f8, accept

Reviews: revision f26f8a8, accept

## Outcome

One record holds every number 0.9.3 is judged by, measured on v0.9.2 on the build host with nothing else building, so 2082 can repeat the same commands after the plan lands.

## Evidence

- The two studies (`sdlc/designs/2026-10-12-0-9-3-iteration-speed.md` and `sdlc/designs/2026-10-13-0-9-3-dependencies-and-dead-code.md`) measured most of these, but each in its own clone and conditions, and neither measured the build folder's size.
- Ian asked for before-and-after numbers on binary size, build folder size, debug and release build time, direct and total dependencies, test time, test count and coverage.

- Starts from: tag `v0.9.2`, a fresh clone on the build host, one build at a time.
- Keeps: the studies' commands where they already measure the number.
- Changes: one record, `sdlc/records/2069-0-9-3-baseline.md`, with a script under `tools/` that prints every number and the command behind it: stripped release binary bytes; build folder bytes after `make test` and after a release build; cold release build; cold dev build of all targets; rebuild after one Rust edit and after one doc edit; direct dependencies, `Cargo.lock` packages and crates compiled; duplicate crate names; `make test`, `make spec` and `make lint` wall times; Rust, Python and spec test counts; outside-in, unit and combined line coverage; CI wall time and runner-minutes for the last ten green runs; tests over one second; LLVM lines; peak release build memory; ignored advisories in `deny.toml`.
- Proof: the script's output in the record, run twice with numbers within a few percent.
- Defers: nothing.

## Amendment, 2026-10-13

Ian ruled that program speed matters more than build speed, so the baseline also measures BioMCP at run time on the build host: `biomcp --version` start-up time with an empty cache and with a large one; and wall time with a warm cache and with the cache off for `get gene BRAF`, `get variant "BRAF V600E"`, `get drug imatinib`, `search article -g BRAF --limit 10` and `search all --gene BRAF --disease melanoma --counts-only`, five runs each, median and spread. The release profile stays thin LTO with one codegen unit; no ticket trades run-time speed for build time.
