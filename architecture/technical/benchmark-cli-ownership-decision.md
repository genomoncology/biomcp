# Benchmark CLI Ownership Decision

## Status

Decided 2026-04-27 (architect ticket 329, pre-v0.8.22 release gate).

## Context

`src/cli/benchmark/` ships a fully-implemented Clap command module
(`BenchmarkCommand::{Run, SaveBaseline, ScoreSession}`) with a public
`async fn run()` dispatcher, a 12-case suite, baseline JSON schema,
regression analysis, child-process isolation, and a pi JSONL session
scorer. Yet `src/cli/mod.rs:5-7` declares `mod benchmark;` under
`#[cfg(test)] #[allow(dead_code)]` and `src/cli/commands.rs` has no
`Benchmark` variant. `target/release/biomcp benchmark --help` exits
with `error: unrecognized subcommand 'benchmark'`. At the same time,
`architecture/technical/cli-decomposition-2026.md` lines 254, 331,
and 340 assert the `biomcp benchmark ...` grammar is a stable public
surface. The architecture doc and the production binary contradict
each other; ticket 327 ranked this the highest-impact open contract
ambiguity blocking v0.8.22.

The ticket scoped three resolutions: ship public CLI, formalize as an
internal harness, or delete the module.

See `.march/survey.md` for the full inventory, dependency map, blast
radius, and evidence.

## Decision

**Option B — Internal/dev regression harness.** `src/cli/benchmark/`
remains compiled only under `#[cfg(test)]`, is not wired into the
production `Commands` enum, is not advertised on any user-facing
surface, and is documented across architecture, code, docs, and
ratchets as an internal harness rather than a shipped CLI.

### Why this option, not the others

**Against public CLI:**

1. `src/cli/benchmark/run/execute.rs:42` calls
   `std::env::current_exe()` to spawn child processes. The benchmark
   regresses BioMCP against itself. It assumes a compiled binary
   already exists; it is not a standalone command for end users.
2. `BenchmarkCommand::ScoreSession` reads `pi`-format JSONL and
   grades agent behavior. `pi` is an internal coding harness; no end
   user produces or consumes pi JSONL session traces.
3. The command has never appeared in `biomcp --help`, the CLI
   reference, the skill catalog, the README, the operator docs, or
   any release note. There is no public surface expectation to
   preserve.
4. Wiring it public would require: operator docs, an API-key
   environment section for the spawned subprocess, help-text
   verification in `spec/surface/cli.md`, an MCP-surface decision
   for `score-session`, and changes to `biomcp list`/`biomcp skill`
   indexes. None of that is in scope for a release-gate decision and
   none of it has demand evidence.

**Against deletion:**

1. The module is fully implemented and decomposed (ticket 325). The
   suite-version pin, baseline JSON schema, regression classifier,
   contract-failure timing budget, and pi-session scorer represent
   real CI value that we want available when v0.9 stabilizes.
2. `tests/benchmark_cli_structure.rs` is an existing ratchet; the
   doc/code/test alignment for deletion would still need 332 and 335
   to do real work, just inverted.
3. Deletion would remove a regression-detection capability we will
   want for v0.9 release prep without producing meaningful surface
   simplification — the module is already invisible to end users.

**For internal harness:**

1. The code already declares `#[cfg(test)]` — current intent is
   internal. Formalizing the declaration is the smallest change.
2. Clap grammar can stay as-is for in-test invocation. No runtime
   surface deletion needed; only the description of what the module
   IS gets corrected.
3. A contract test (ticket 335) can pin the harness invariant
   bidirectionally: the production `Commands` enum must NOT contain
   `Benchmark`, AND no architecture or public docs claim
   `biomcp benchmark ...` is a public surface. This prevents drift
   in either direction.
4. Honest: the harness exists to regression-test BioMCP, not to
   serve operators. Calling it that ends the contradiction.

## Target Architecture

### Module surface

- **`src/cli/mod.rs:5-7`** keeps `#[cfg(test)] #[allow(dead_code)]
  mod benchmark;`. The follow-up build/quickfix ticket adds an
  inline comment naming this explicitly as an internal regression
  harness so future readers do not retry the wiring.
- **`src/cli/benchmark/mod.rs`** keeps the dispatcher but renames
  `BenchmarkCommand` → `InternalBenchmarkCommand` to remove the
  visual ambiguity that "BenchmarkCommand" implies public clap
  exposure. Each `Subcommand` doc-string is prefixed
  `[internal harness]` so any accidental help render reflects the
  truth. The module-level `//!` header is updated to start
  `Internal regression harness ...`.
- **`src/cli/commands.rs`** stays unchanged. No `Benchmark` variant
  is added. The production `Commands` enum continues to reject
  `biomcp benchmark`.

### Tests / ratchets

- **`tests/benchmark_cli_structure.rs`** keeps the file-layout,
  doc-header, and 700-line assertions. Ticket 335 extends it (or
  adds a sibling integration test) with the bidirectional contract
  assertions:
  1. `Commands` enum (compile-time check via parsing
     `src/cli/commands.rs` source or via a small in-crate test)
     contains no `Benchmark` variant.
  2. `architecture/technical/cli-decomposition-2026.md` and
     `architecture/technical/benchmark-cli-ownership-decision.md`
     contain no `biomcp benchmark <subcommand>` claims framed as
     public grammar; the words "internal harness" or equivalent
     appear next to every benchmark mention.
  3. `target/release/biomcp benchmark` (when a release binary is
     present) exits non-zero with `unrecognized subcommand`. This is
     opt-in — not every gate has a release binary on disk — but the
     test is an additional contract for the release lane.

### Architecture doc

- **`architecture/technical/cli-decomposition-2026.md`** drops the
  three "biomcp benchmark public grammar" claims (lines 254, 331,
  340) and the "benchmark cluster decomposition is forward work"
  framing. Ticket 332 owns the full edit; this decision doc carries
  only a short pointer at the relevant section. The exact edits 332
  must make are listed below under "Instructions for ticket 332".

### Cross-surface claims

- `biomcp --help` MUST NOT list `benchmark`.
- `biomcp benchmark --help` MUST exit with `unrecognized
  subcommand`.
- `biomcp list`, `biomcp skill`, `biomcp suggest`, `biomcp discover`,
  `biomcp health`, README, `docs/`, `architecture/ux/cli-reference.md`,
  and `spec/surface/cli.md` MUST NOT mention `biomcp benchmark`.
- `architecture/technical/cli-decomposition-2026.md` MUST mention
  benchmark only as a decomposed-and-internal harness with a
  pointer to this decision doc.

### Invariants the new architecture enforces

| Invariant | Pinned by |
|---|---|
| Production binary does not route `biomcp benchmark` | ticket 335 contract test (Commands enum, optional release-binary check) |
| Architecture doc never re-asserts `biomcp benchmark` is a public surface | ticket 335 contract test (string scan) |
| `src/cli/benchmark/` stays under 700-line cap and decomposed layout | existing `tests/benchmark_cli_structure.rs` |
| Module doc, top enum name, and clap doc-strings name it as an internal harness | ticket NEW (code-side rename) |

## Follow-up tickets

Three child tickets cover the alignment work. None block the
architect ticket itself; only the runtime-wiring ratchet (335)
requires the others to land first.

| ID | Name | Flow | Dependencies | Owner zone |
|---|---|---|---|---|
| NEW | Mark `src/cli/benchmark/` as internal harness in code | quickfix | 329 | code (`src/cli/mod.rs`, `src/cli/benchmark/mod.rs`) |
| 332 | Refresh stale `cli-decomposition-2026.md` | quickfix | 329 | architecture doc |
| 335 | Add benchmark runtime-wiring ratchet | quickfix | 329, 332, NEW | tests / contract |

The new ticket is small enough to be a quickfix. It does only:

1. Update `src/cli/mod.rs:5-7` to add an explicit
   `// Internal regression harness; not wired into production CLI.
   // See architecture/technical/benchmark-cli-ownership-decision.md.`
   comment block alongside the `#[cfg(test)] #[allow(dead_code)]`
   declaration.
2. Update `src/cli/benchmark/mod.rs` `//!` header to start
   `//! Internal regression harness for BioMCP CLI ...`.
3. Rename `BenchmarkCommand` → `InternalBenchmarkCommand` and
   prefix each `Subcommand` doc-comment with `[internal harness]`.
4. Update inline call sites within `src/cli/benchmark/` to the new
   name (compile gate proves completeness).
5. Confirm `cargo test --lib` and `make spec-pr` stay green.

The rename intentionally happens before ticket 335 so the contract
test references the stable post-rename symbol.

## Instructions for ticket 332

Ticket 332 already exists, is `flow: quickfix`, depends on 329, and
is in `ready` state. It must change `cli-decomposition-2026.md` as
follows for the benchmark portion (other portions of the refresh —
shipped slice numbers, deleted-flat-file inventory, residual cap
table — stay as 332 already scoped them):

1. **Section 6 (lines 223-262)**: rewrite the "Benchmark cluster
   decomposition" section header to "Benchmark internal harness
   (decomposed; not a public CLI)". Drop the "stable facade"
   framing and replace it with: "The `src/cli/benchmark/` tree is
   compiled only under `#[cfg(test)]` and is not wired into the
   production `Commands` enum. Ownership is fixed by
   `architecture/technical/benchmark-cli-ownership-decision.md`."
2. **Line 254** ("`biomcp benchmark ...` command grammar remains
   unchanged"): delete this assertion outright. Replace with: "The
   internal harness exposes `Run`, `SaveBaseline`, and
   `ScoreSession` only inside `cargo test`; production CLI grammar
   is unaffected."
3. **Lines 308 (slice-plan table row 7)**: keep the row but mark
   the slice "shipped (ticket 325)" and update the proof column to
   "internal harness layout + `cli-module-decomposition.md` cap";
   drop the "benchmark help/output/schema stability" framing.
4. **Lines 331 and 340 (proof contract bullets)**: remove the
   `benchmark` examples from the public canary/proof lists. The
   benchmark area is no longer a public-surface refactor target.
5. **Add a one-line pointer near the top of the document**:
   "Benchmark CLI ownership is fixed by
   `architecture/technical/benchmark-cli-ownership-decision.md`;
   benchmark sections below describe the internal harness only."

These edits MUST land in 332 (not 329) so the architect ticket does
not silently rewrite migration plans it is not scoped to own.

## Instructions for ticket 335

Ticket 335 already exists, is `flow: quickfix`, depends on 329 and
332, and is in `ready` state. The "internal harness" branch of its
scope applies. The ticket must:

1. Add a Rust integration test (extending
   `tests/benchmark_cli_structure.rs` or a new
   `tests/benchmark_cli_contract.rs`) that asserts:
   - `src/cli/commands.rs` source contains no `Benchmark(` or
     `Benchmark {` enum-variant token.
   - `architecture/technical/cli-decomposition-2026.md` does not
     contain the substring `biomcp benchmark ` followed by `...` or
     a subcommand word, except inside an explicit
     "internal harness" sentence.
   - `architecture/technical/benchmark-cli-ownership-decision.md`
     exists and contains the words "internal harness" near the top.
2. Optionally add a release-binary canary that, when
   `target/release/biomcp` exists, runs it with `benchmark --help`
   and asserts non-zero exit with `unrecognized subcommand` in
   stderr. Skip cleanly if no release binary is present.
3. Update the structure test names/comments to call the harness
   "internal" and not a public CLI surface.
4. Add this ticket's contract to the v0.8.22 release-gate test
   matrix in CHANGELOG/release notes (or via the release-cut
   ticket) so a future contributor cannot quietly re-publish
   benchmark without failing the gate.

This pins the contract bidirectionally: production binary cannot
gain a `Benchmark` variant without breaking the test, and the
architecture doc cannot quietly re-claim public benchmark grammar
without breaking the test.

## ScoreSession — debt note

`BenchmarkCommand::ScoreSession` shares a namespace with
`Run`/`SaveBaseline` only because both live under
`src/cli/benchmark/`. Conceptually it is a separate dev tool:
it grades pi JSONL sessions, not BioMCP CLI calls. Splitting it out
(e.g. `src/cli/dev_tools/score_session/`) is reasonable cleanup but
is **not** in scope for v0.8.22. Survey question 2 raised this; the
internal-harness decision does not block on it. Track it in the
"Post-v0.8.22 architecture cleanup" frontier section.

## `benchmarks/v*.json` baseline path

The harness's `SaveBaseline` writes to `benchmarks/v<VERSION>.json`
and `discover_latest_baseline()` reads from there. No such file
ships in the repo today (only `benchmarks/bioasq/`). Keeping the
path implicit is fine for v0.8.22 — it only matters when a
maintainer first runs `cargo test ... save-baseline`. If a CI
script ever invokes the harness, the path SHOULD be documented in
that script's README. No ticket needed at this time.

## Operator and agent impact

- **End users:** zero impact. `biomcp benchmark` was never
  reachable; nothing changes for them.
- **Contributors:** `cargo test --lib` continues to compile and run
  the harness. The rename to `InternalBenchmarkCommand` is a
  one-line `git grep` chase if any out-of-tree fork imports the
  symbol; in-tree there are no production callers.
- **March / CI agents:** the contract test in 335 turns the
  internal-vs-public ambiguity into a hard gate. Any future ticket
  that tries to wire benchmark public will fail `make check` until
  it also revises this decision doc and 335's contract test.
- **Release reviewers:** the architecture doc no longer claims a
  surface that does not exist. The 327 review's `#2` blocker is
  resolved.

## Out of scope for this decision

- Actually deleting the module (rejected).
- Splitting `ScoreSession` into a separate dev-tool surface
  (recorded as post-v0.8.22 cleanup).
- Documenting the `benchmarks/v*.json` baseline path (not needed
  for v0.8.22; no current consumer).
- Rewriting `cli-decomposition-2026.md` body — that is ticket 332.
- Adding the runtime-wiring contract test — that is ticket 335.
- Adding the rename / module comment — that is the new follow-up
  build/quickfix ticket.

## References

- `.march/survey.md` — full investigation, evidence, blast radius.
- `architecture/technical/cli-decomposition-2026.md` — currently
  stale; ticket 332 will refresh.
- `architecture/technical/cli-module-decomposition.md` — durable
  decomposition contract; unchanged.
- `tests/benchmark_cli_structure.rs` — existing layout ratchet;
  ticket 335 extends.
- `src/cli/mod.rs:5-7` — `#[cfg(test)]` declaration to be
  comment-clarified.
- `src/cli/benchmark/mod.rs` — `BenchmarkCommand` to be renamed
  `InternalBenchmarkCommand`.
- Tickets 325 (benchmark dispatcher not wired), 327 (structure
  ratchet misses runtime wiring), 332 (stale doc refresh), 335
  (runtime-wiring ratchet).
