# Harden: Command-level article spec runtime

## Scope Disposition

Ticket 505 is a measurement-only Rust spike. It explicitly forbids implementing or editing production/test code. It compared two frozen revisions, retained evidence, and selected a future optimization boundary; it did not produce an optimized implementation to decompose.

The generic harden instructions assume a Zig implementation and a downstream spike graph. Neither exists here: `~/workspace/planning/mole/spike-plan.md` is absent, the ticket names no downstream spike IDs, and its listed dependencies (504 and 506) are upstream. The only downstream consumer is the future same-repository build ticket described by the decision packet.

For that reason, hardening is deliberately a no-op for product code. Inventing a public fixture-pacing API, exposing private rate-limit internals, or committing the temporary profiling harness would violate the ticket and add unnecessary abstraction.

## Decomposition

No code was extracted because ticket 505 added no code.

The repository already has the required library/CLI split:

- `src/lib.rs` is Cargo's `biomcp_cli` library target.
- `src/main.rs` is an 87-line binary wrapper. It initializes tracing, parses arguments through `biomcp_cli::cli`, delegates MCP modes to `biomcp_cli::mcp`, and writes the returned output/exit status.
- `src/sources/rate_limit.rs` owns the measured pacing behavior inside the library. `RateLimitPolicy`, `RateLimiter`, and middleware wiring remain crate-private because no external consumer needs them.

There is no CLI monolith from this spike, no new shared type, and no reader/writer implementation to move. The temporary wrapper and analyzer were measurement apparatus, not product assets; they remain private March evidence and must not become a supported library.

## Public API

### Decision

Ticket 505 adds **no public Rust API**. Its downstream contract is evidence and behavior, not callable implementation.

Existing supported library entry points remain unchanged:

```rust
use biomcp_cli::{cli, mcp};
```

A same-repository implementation should work behind the existing HTTP/rate-limit integration rather than introduce an external API. In particular, downstream code must not import a new `profile_505` module, shell out to `biomcp`, or copy the removed wrapper/analyzer.

### Evidence consumption example

A downstream planning or benchmark tool can consume the tracked compact data directly instead of invoking the CLI:

```rust
use serde_json::Value;

let raw = std::fs::read_to_string(
    "architecture/experiments/505-profile-command-level-article-spec-runtime-before-choosing-another-optimization/results.json",
)?;
let profile: Value = serde_json::from_str(&raw)?;
```

This is a data-file contract, not a new BioMCP API. The authoritative interpretation and behavior floor remain in `exploit.md` and `optimize.md` beside that JSON file.

### Contract for the next build

The future build should add an explicit fixture-only signal at the existing HTTP rate-limit boundary. It must skip live-upstream pacing only for test-owned local source-base overrides, preserve real source/API-key intervals, contain fixture environment lifetime, and retain:

- article `23 passed, 3 skipped` behavior and expected command exits from the frozen comparison;
- resolver ordering and clean no-`--pdf` miss behavior;
- XML, HTML, PDF, PMC OA, Figshare asset/sibling, cold-storage retry, provenance/license, JSON, saved-file, and JATS Markdown contracts;
- fixture ownership, standalone renderer fallback, cleanup, cache/XDG isolation, and offline routine execution.

## Build System

This is Cargo, not Zig. No `build.zig` exists or is needed. Cargo already discovers both target classes:

- library: package `biomcp-cli`, Rust crate name `biomcp_cli`, source `src/lib.rs`;
- binaries: `biomcp` at `src/main.rs` and `biomcp-cli` at `src/main_biomcp_cli.rs`.

An external Rust spike that genuinely needs the existing supported library can use a path dependency:

```toml
[dependencies]
biomcp-cli = { path = "../biomcp" }
```

and import it as:

```rust
use biomcp_cli::{cli, mcp};
```

No manifest change was made because Cargo metadata already reports one library and two binary targets. The expected follow-up is in this same crate and should not create a cross-crate dependency merely to alter an internal pacing policy.

## Regression Check

### Performance

Harden changed no files under `src/`, `scripts/`, or `spec/`, and changed no Cargo manifest/lock/build file. Therefore decomposition adds exactly zero runtime or build overhead. The full retained frozen benchmark remains unchanged:

| Suite | Baseline median | Candidate median | Candidate change |
|---|---:|---:|---:|
| Article-only | 335081 ms | 332757 ms (332983 ms with outer lifecycle) | -0.69% (-0.63% inclusive) |
| Full routine | 567221 ms | 707933 ms | +24.81% regression |

The retained evidence contains 12 successful suite samples, 696 normal command rows, and 52 equal-method diagnostic rows. Ticket 502 still fails its 60% target. The candidate's full-routine regression is not excused.

The frozen worktrees, shared binary, fixture roots, and wrapper were removed as required by ticket 505, so an exact rerun is intentionally impossible. Running current `main` would not reproduce the same frozen-revision/same-binary comparison. Zero regression from hardening is instead proven by the empty product/test diff and unchanged retained result data.

### Correctness

Fresh standard gates after the harden audit passed:

- `make lint`
- `make test`: 2385 Rust tests passed (28 skipped), 323 Python tests passed, strict documentation build passed
- `make spec`: 133 passed, 6 skipped, plus 30 isolation-contract tests passed

The current routine count includes changes merged after the frozen experiment and is not substituted for its baseline. The frozen comparison remains article `23 passed, 3 skipped` and full `134 passed, 5 skipped` plus 29 canary passes for every sample, with identical expected command-exit signatures.

## Reusable Assets

Downstream work inherits these concrete assets:

1. **Compact profile data:** `results.json` with suite accounting, command rankings, and diagnostic counts.
2. **Decision packet:** `exploit.md` with frozen SHAs, binary identity, cache policy, behavior floor, root-cause class, and the smallest build boundary.
3. **Optimization analysis:** `optimize.md` with measured and projected passes clearly separated, convergence, and rejected unsafe/broad alternatives.
4. **Measurement method:** three cold article samples per revision, one absolute same binary per before/after comparison, command-level accounting, and at least one equal-method diagnostic.
5. **Regression contracts:** exact article behavior, source-rate policy preservation, fixture ownership/cleanup, and complete-routine regression control.
6. **Existing crate boundary:** Cargo library target `biomcp_cli` plus an 87-line `biomcp` wrapper; no new public surface is required.

Not reusable as public code: the private March wrapper, analyzer, raw traces, removed binary, fixture roots, and temporary worktrees. Promoting them would create machine-specific test apparatus rather than a stable library.
