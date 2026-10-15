# 2070 — Delete build.rs and build once per ticket

Status: COMPLETE.

Milestone: 0.9.3

Reviews: revision 0fcfebb, accept

## Outcome

Editing a doc, a spec page or a Python test recompiles nothing, and a ticket's gates compile BioMCP once in the dev profile.

## Evidence

- An empty `build.rs` makes cargo rescan the package and recompile the crate after any file change: a spec page touch costs 175 s, and 2.7 s with the file gone (measured, `sdlc/designs/2026-10-12-0-9-3-iteration-speed.md`, step 1).
- `make test` and `make spec` build a second `spec` profile binary on top of the dev build, 253 s after a Rust edit, though both lanes pass on the dev binary (step 2).

- Starts from: `sdlc/designs/2026-10-12-0-9-3-iteration-speed.md`, steps 1 and 2.
- Keeps: the release profile for `make release-gate` and the release workflow.
- Changes: delete `build.rs`; point the Python and spec lanes at the dev test build's binary; drop the `spec` profile from routine gates; set `CARGO_INCREMENTAL=1` in the documented targeted-test loop only.
- Proof: on the build host, a spec-page touch then `cargo build --all-targets` compiles nothing; `make test` and `make spec` after a Rust edit print one `Compiling biomcp-cli`; their wall times recorded before and after; a CI run that touches only a spec page compiles nothing.
- Defers: CI job structure (2075).

Executed on the branch, yellow, both locks held, head `380c3bb68` (the rebase onto main `0df0214fa` — the landed 2093 — changed only the changelog, the dev-cycle ticket set, and Cargo.lock's rustls bump, none of which touch the build.rs deletion or the lane wiring, so every count below stands on the rebased head `1ee390af8` as well):

Red state (before, base `ca54283f0`, build.rs present):

- Spec-page touch (`spec/entity/gene.md`) then `tools/with-build-identity cargo build --all-targets`: 1 `Compiling biomcp-cli`, 159.4 s (the 2069 baseline record's doc-edit cluster: 156.8-176 s).
- `make prepare-test` after a Rust-file touch: 2 `Compiling biomcp-cli` lines — the spec profile (`Finished spec profile in 4m18s`) plus the dev archive — 424.7 s wall. That is the lanes-build-twice red state.

After state:

- Spec-page touch then `tools/with-build-identity cargo build --locked --no-default-features --all-targets`: 0 Compiling, 0.41 s. Touching a Python test and `tools/rust-source-size-inventory.json` together: 0 Compiling, 0.42 s. (A bare `cargo build --all-targets` without `--no-default-features` requests a different feature set and always compiled; the gates' feature set is the honest comparison.)
- `make prepare-test` after a Rust-file touch: exactly 1 `Compiling biomcp-cli`, 174.3 s (was 2 units and 424.7 s).
- `make test` after a Rust edit: exactly 1 `Compiling biomcp-cli`. Green full run on the final state with fresh artifacts: 657.2 s (baseline record: 646.3 / 640.0 s), 4223 Rust tests passed, 33 skipped, 1252 Python tests passed, 3 skipped. The known load-flaky `rmcp_client_contract::raw_and_typed_article_query_validation_converges_before_provider_work` failed three first attempts at its 60 s/120 s boundary and passed on rerun, the exact signature the 2069 baseline record documents for v0.9.2; CI never shows it.
- `make spec` after a Rust edit: exactly 1 `Compiling biomcp-cli` (the preparer's dev bin+example build), green, 786.5 s (baseline record with nothing to compile: 657.2 / 668.3 s).
- `make lint` green after the 2093 rebase: 146.6 s (the rustls advisory had been the only lint failure; baseline record lint: 195.9 / 196.5 s).

CI proofs:

- Rebased head `1ee390af8`: run 38583882747, completed success, 26m02s.
- Spec-page-only commit `77b48f822` (one prose sentence in `spec/entity/gene.md`): run 38584319137, completed success, 25m20s. Its canonical-gates log: the lint gate (clippy over the whole crate) and the specification gate compile nothing (0 Compiling, 0 `Checking biomcp-cli`); the test gate shows 2 `Compiling` lines (`biomcp-cli`, `biomcp-mcp-contract-client`) that are identical to the prior code-changing run's — the per-commit build-identity stamp, not the page — with the archive finishing in 1m51s and the contract build after it in 0.37s. Before this ticket, the build-script scan recompiled the crate in both profiles on every CI run.
