---
flow: build
priority: 7
deps: [1161]
---

# Expose drug section commands in every card surface

BioMCP now builds drug-card recovery, unloaded-section, aggregate, and related
commands through one internal discovery projection. Single and batch JSON and
Markdown therefore expose the same ordered, executable suggestions. Commands
retain the resolved drug identity and effective region, use the shared shell
quoting boundary, deduplicate exact bytes before the ten-command cap, and do
not cause additional provider work.

The implementation preserves existing parsing, acquisition, section outcome,
related-command, MCP schema, tool inventory, search, and interaction-report
behavior. WHO still excludes unsupported safety and shortage commands, and
generic batches retain US acquisition behavior.

## Evidence

Independent SOL design review accepted the focused contract and level 2 Luna
implementation route. Independent SOL code review first rejected missing
production batch, MCP, command-execution, and exact pure-projection evidence.
The next review required complete exact categorized and flattened matrices.
The final review accepted exact owner projections plus a mutation-sensitive
unit proof of the private deduplication boundary; current candidate command
families are structurally disjoint, so no test-only runtime seam was added to
manufacture an impossible product state.

Production contracts prove exact single, multi-section, and two-item batch
JSON/Markdown agreement, stable input order and resolved identities, unchanged
provider request counts, raw and typed MCP inheritance, unchanged tool/schema
inventory, regional recovery, representative emitted-command execution, and a
provider-sourced hostile identity whose emitted shell command preserves exact
bytes without creating a shell side effect. Pure tests pin all explicit
sections, `all`, unavailable and degraded recovery, every region, sparse and
blank branches, deduplication, and the nine/ten/over-ten candidate boundaries.

On the accepted revision, `make lint`, `make test`, and `make spec` passed under
Node 22. The shared development host used eight nextest workers: all 3,399 Rust
tests passed; 955 Python contracts passed with three intentional skips; strict
documentation and every offline executable and static specification group
passed. `cargo package --list --locked --offline --no-verify` reports exactly
1,300 paths, and `git diff --check` passes.
