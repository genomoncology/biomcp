---
flow: build
priority: 5
deps: []
---

# Correct reversed search commands with the canonical form

BioMCP now recognizes all fifteen exact reversed search families and reports a
copyable `biomcp search <entity>` command without accepting or executing the
reversed form. The recovery preserves argument bytes and ordering through
lossless POSIX-shell quoting, uses CommonMark-safe adaptive code spans, and
retains the established human, JSON, raw-MCP, and typed-MCP boundaries.

The detector is a bounded lexical preflight. Invalid or suppressed reversals
for the three external-subcommand families receive a fixed no-echo error and
cannot fall through to provider or cache work; the other twelve families keep
their original Clap diagnostics. Subprocess and raw-MCP contracts prove the
correction performs no provider request and creates no managed state. No alias,
dependency, public schema, or package path was added.

## Evidence

A fresh SOL design review rejected the original draft over CommonMark padding,
ambiguous recovery bounds, underestimated complexity, stale dependencies, and
an overbroad no-work claim. The ticket was amended with exact inclusive bounds,
a three-outcome classifier, surface-specific envelopes, and observable no-work
proof, then accepted as Level 3.

Independent code review found two material escape paths in succession. First,
the `gene`, `drug`, and `variant` external-subcommand catchalls could accept the
reversed words before recovery. Second, invalid or bounds-suppressed candidates
could still fall through those catchalls. Both findings were remediated, and
the reviewer accepted exact revision `5c89d3e7` after reproducing poisoned
provider cases, absent cache state, all fifteen families, native stdio, raw MCP
over stdio and HTTP, safe shell rendering, CommonMark fencing, and boundary
behavior.

On that accepted revision, `make lint`, `make test`, `make spec`, and
`make full-feature-check` passed under Node 22. The shared host used eight
nextest workers: all 3,450 Rust tests passed with 31 intentional skips; 955
Python contracts passed with three intentional skips; strict documentation and
every offline executable, isolation, fixture-ownership, and static
specification group passed. All-feature Clippy, six AlphaGenome behavior tests,
the optimized all-feature build, and PNG/SVG/terminal artifact smoke passed.

The first complete test attempt encountered the cache manager's configured
low-space eviction after the host fell below its ten-percent free-space floor;
one cache-metadata assertion failed for that reason. Reclaiming only
reproducible build and routine-test artifacts restored 101 GiB free. The exact
test then passed in isolation and the complete test gate passed on its rerun.

`src/cli/shared.rs` remains within its 700-line cap at 696 lines,
`src/mcp/shell.rs` remains at its existing 2,149-line allowance,
`cargo package --list --allow-dirty --locked --offline --no-verify` reports
exactly 1,300 paths, and `git diff --check` passes. No Oracle skill was used.
