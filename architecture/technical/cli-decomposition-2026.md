# CLI Decomposition 2026 Retrospective and Residual Plan

## Status

This document is no longer the live plan for tickets 319-325. Those CLI
module-decomposition slices have shipped, and the durable decomposition contract
now lives in `architecture/technical/cli-module-decomposition.md`.

Keep this file as a dated 2026 migration record plus a short residual-over-cap
plan. Do not use it as the steady-state architecture contract.

Benchmark CLI ownership is fixed by
`architecture/technical/benchmark-cli-ownership-decision.md`: the benchmark tree
is an internal regression harness compiled under `#[cfg(test)]`, not a public
CLI surface. Benchmark sections below describe that internal harness only.

## What the 2026 decomposition push fixed

Survey ticket 309 identified four root issues:

1. no reusable CLI module-decomposition contract or area ratchets
2. mixed planning, dispatch, formatting, runtime helper, and test ownership in
   large flat CLI files
3. inline or flat test ownership inflating runtime modules
4. stale public-surface proof language that was copied between slices instead of
   tied to each command family's real exposure

Tickets 319-325 moved the named oversized areas into facade modules plus focused
siblings, added area structure ratchets, and kept the public command surfaces
unchanged where those surfaces are actually shipped.

## Completed decomposition slices

The files in this table are historical inputs to the 2026 migration. They are
not current oversized flat-file inventory.

| Ticket | Area | Historical input | Shipped shape | Proof / ratchet |
|---|---|---|---|---|
| 319 | Search-all decomposition | `src/cli/search_all.rs` | `src/cli/search_all/{mod,plan,dispatch,links,format}.rs` plus sidecar tests | `tests/search_all_cli_structure.rs`; search-all help/spec behavior preserved |
| 320 | Health decomposition | `src/cli/health.rs` | `src/cli/health/{mod,catalog,http,local,runner}.rs` plus split tests | `tests/health_cli_structure.rs`; health output/help/local-source coverage preserved |
| 321 | Suggest decomposition | `src/cli/suggest.rs` | `src/cli/suggest/{mod,routes,extract,patterns}.rs` plus sidecar tests | `tests/suggest_cli_structure.rs`; offline routing and parseability preserved |
| 322 | Skill decomposition | `src/cli/skill.rs` | `src/cli/skill/{mod,assets,catalog,install}.rs` plus sidecar tests | `tests/skill_cli_structure.rs`; read-only catalog and install workflow preserved |
| 323 | List decomposition | `src/cli/list.rs` | `src/cli/list/{mod,helpers,molecular,clinical,literature}.rs` plus sidecar tests | `tests/list_cli_structure.rs`; list/reference docs contracts preserved |
| 324 | Article CLI test split | `src/cli/article/tests.rs` | `src/cli/article/tests/{mod,help,exact_lookup,json,filters}.rs` | `tests/article_cli_tests_structure.rs`; article CLI coverage preserved with isolated test domains |
| 325 | Benchmark harness decomposition | `src/cli/benchmark/run.rs`, `src/cli/benchmark/score.rs` | `src/cli/benchmark/run/` and `src/cli/benchmark/score/` subtrees | `tests/benchmark_cli_structure.rs`; internal harness layout and file cap preserved |

The current code also preserves the durable facade pattern documented in
`cli-module-decomposition.md`: each area owns a stable `mod.rs` boundary, focused
ownership-zone files, sidecar tests, and an area-specific structure ratchet.

## Benchmark internal harness

Ticket 329 decided the benchmark module is an internal/dev regression harness.
The `src/cli/benchmark/` tree remains compiled only under `#[cfg(test)]` and is
not wired into the production `Commands` enum. Its Clap-shaped `Run`,
`SaveBaseline`, and `ScoreSession` subcommands exist only for in-test harness
use; production CLI grammar is unaffected.

Implications for this document:

- benchmark is not part of the public CLI preservation proof matrix
- benchmark help/output wording is not a user-facing canary
- ticket 325 is complete as an internal harness decomposition, not pending public
  command work
- ticket 335 owns the follow-up runtime-wiring ratchet that prevents the
  architecture and production binary from drifting apart again

## Residual over-cap inventory

Ticket 327's release-readiness review found that the named 319-325 surfaces are
healthy, but the global `src/cli/**/*.rs <= 700` rule is not yet enforced across
all CLI files. The remaining over-cap files are:

| File | Lines at 327/332 refresh | Residual owner |
|---|---:|---|
| `src/cli/drug/tests.rs` | 869 | Ticket 334 global line-cap ratchet allowlist |
| `src/cli/trial/tests.rs` | 797 | Ticket 334 global line-cap ratchet allowlist |
| `src/cli/cache.rs` | 759 | Ticket 334 global line-cap ratchet allowlist |
| `src/cli/article/session.rs` | 722 | Ticket 334 global line-cap ratchet allowlist |
| `src/cli/article/dispatch.rs` | 722 | Ticket 334 global line-cap ratchet allowlist |
| `src/cli/variant/dispatch.rs` | 702 | Ticket 334 global line-cap ratchet allowlist |

Ticket 334 is the absorb-back path: it adds a global scan with an explicit,
dated allowlist and follow-up ownership for every residual over-cap file. This
retrospective does not decompose those files and should not grow a parallel
residual-ticket plan.

## Residual decomposition guidance

Future work on the residual files should follow the durable contract, not the
superseded slice text that used to live in this file:

- keep the stable in-crate facade path for the touched command family
- split by ownership zone, not by arbitrary line count
- keep public command grammar/help/spec behavior unchanged unless the ticket
  explicitly changes behavior
- move large test ownership into focused sidecars when test size is the problem
- add or update the area ratchet in the same ticket that decomposes the area
- remove allowlist entries from ticket 334's ratchet when a file comes back under
  the cap

Likely ownership zones by residual file:

| File | Likely split axis |
|---|---|
| `src/cli/drug/tests.rs` | help/parse tests, search/filter tests, detail/pivot tests, JSON/render tests |
| `src/cli/trial/tests.rs` | help/parse tests, search/filter tests, detail/eligibility tests, JSON/render tests |
| `src/cli/cache.rs` | command routing, path/config inspection, cleanup execution, render helpers |
| `src/cli/article/session.rs` | session parsing, loop prevention, ranking context, persisted-state helpers |
| `src/cli/article/dispatch.rs` | request construction, source dispatch, exact lookup, response shaping |
| `src/cli/variant/dispatch.rs` | identifier parsing, query planning, source dispatch, response shaping |

These are orientation notes only. Any actual decomposition should be filed as a
build-flow ticket with its own focused design/proof matrix.

## Public-surface proof contract for future CLI decompositions

For future behavior-preserving CLI decompositions, use the proof contract in
`architecture/technical/cli-module-decomposition.md`. In short:

- run focused Rust tests for the touched command family
- run clippy/tests/spec gates required by the ticket
- preserve `biomcp --help` and the affected shipped command family's long help
- reuse existing specs/contracts instead of adding brittle snapshots
- keep docs/help/examples in sync when the ticket changes shipped behavior

Current shipped public-surface examples:

- `health`, `list`, and `search all` → `spec/surface/cli.md`
- `suggest` and `skill` → `spec/surface/discover.md`
- list/docs alignment → `tests/test_public_search_all_docs_contract.py` and
  `tests/test_public_skill_docs_contract.py` when relevant

Do not add benchmark to that public-surface list unless a future architecture
and build ticket deliberately ships it as a production command and updates the
benchmark ownership decision accordingly.

## Alignment with team goals and frontier

The completed decomposition work and the residual ticket 334 path align with:

- **Goal G6: quality over features** — architecture ratchets and review-sized
  files reduce regression risk without inventing new functionality
- **Frontier: Quality Ratchet Absorptions** — the original CLI 700-line cap gap
  was absorbed for tickets 319-325 and is being generalized by ticket 334
- **FAQ #18** — health decomposition preserved coverage for every
  readiness-significant local source
- **FAQ #9** — list/help-facing tickets kept docs-contract tests in the proof
  matrix when they touched command-reference surfaces

The 2026 decomposition batch is complete for its named targets. Remaining work is
limited to the residual over-cap inventory above and the global ratchet that will
own it.
