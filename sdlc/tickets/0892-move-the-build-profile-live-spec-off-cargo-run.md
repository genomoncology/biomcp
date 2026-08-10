---
flow: build
priority: 5
---
# Remove Cargo execution from routine specification pages

## Done when

No routine spec page or fixture helper invokes cargo. The runner prebuilds
each required artifact once and passes explicit paths to:

- the feature-on BioMCP CLI;
- a separate no-default-features CLI;
- MCP examples or helper binaries used by spec/surface/mcp.md and
  spec/fixtures/run-section-outcome-mcp.sh;
- any other executable discovered by the inventory.

The feature-on and feature-off artifacts use distinct output paths so one does
not overwrite the other or contend for Cargo's shared lock while specs run.

## Proof required

- build-profile-live.md consumes the supplied feature-off binary and still
  proves the intended feature difference;
- MCP pages consume prebuilt examples;
- a source ratchet scans routine spec Markdown and helper scripts and rejects
  cargo invocation;
- one runner test proves missing or stale artifact paths fail clearly rather
  than falling back to cargo or an installed biomcp;
- routine spec results remain unchanged.

Live provider smoke pages may consume the same prebuilt release artifact but
must not build it themselves.

## Authorized test changes

Design commits may restate scripts/run-specs.sh, Makefile, the named spec
pages/helpers, artifact-path environment contracts, and runner tests. No
product src change belongs here.

The src line ceiling may not rise.
