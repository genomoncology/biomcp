---
flow: build
priority: 5
---
# Build routine specification artifacts once

## Done when

The routine-spec runner has one explicit preparation phase. That phase builds
each executable test artifact once and passes explicit paths to:

- the feature-on BioMCP CLI;
- a separate no-default-features CLI;
- MCP examples or helper binaries used by spec/surface/mcp.md and
  spec/fixtures/run-section-outcome-mcp.sh;
- any other executable discovered by the inventory.

The feature-on and feature-off artifacts use distinct output paths so one does
not overwrite the other or contend for Cargo's shared lock while specs run.

Filtered Rust tests are compiled with `cargo test --no-run` during preparation;
the pages execute the resulting test binaries directly with their existing
filters. `cargo metadata --no-deps` and `cargo tree --locked` are legitimate
evidence in `spec/surface/build-profile.md`. Generate each once during
preparation and pass its captured, command-labelled output to the page. Do not
replace these proofs with source-text guesses. During page execution and
fixture-helper execution, forbid build-inducing `cargo run`, `cargo build`,
`cargo rustc`, `cargo test`, or equivalent nested builds.

## Proof required

- build-profile-live.md consumes the supplied feature-off binary and captured
  metadata/tree evidence and still
  proves the intended feature difference;
- MCP pages consume prebuilt examples and filtered tests execute prebuilt test
  binaries;
- a source ratchet parses executable shell blocks and helper commands, ignores
  prose and expected-output literals, and rejects build-inducing Cargo commands
  outside the runner's preparation phase;
- the ratchet explicitly permits only the preparation-owned metadata/tree
  capture and artifact builds, so a new nested compilation path fails loudly;
- one runner test proves missing or stale artifact paths fail clearly rather
  than falling back to cargo or an installed biomcp;
- routine spec results remain unchanged.

Live provider smoke pages may consume the same prebuilt release artifact but
must not build it themselves.

## Authorized test changes

Design commits may restate scripts/run-specs.sh, Makefile, the named spec
pages/helpers, filtered-test invocation, artifact/evidence path environment
contracts, and runner tests. Existing assertions that actually prove Cargo
metadata, dependency-tree, and filtered-test behavior may be restated to
consume preparation output; they must not be deleted. No product src change
belongs here.

The src line ceiling may not rise.
