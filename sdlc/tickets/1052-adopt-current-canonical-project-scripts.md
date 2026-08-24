---
flow: build
priority: 30
waits-on: ["botassembly/sdlc/0153"]
---
# BioMCP adopts the current canonical project scripts

`factory doctor` reports all five of this repository's `sdlc/project` scripts as drifted from SDLC's deployed canonical set. The files are older canonical copies, not a BioMCP-specific lifecycle fork. Canonical changes now include owned-orphan recovery, safe moved-main settlement, optional project deployment, withdrawal receipts, and early validation of malformed `opens:` declarations.

Adopt the deployed canonical files through SDLC's provenance-safe propagation command. BioMCP has no project-specific lifecycle behavior to preserve and no deployment hook to add.

## What done looks like, observably

- `sdlc/project/tasks`, `before`, `success`, `failure`, and `health` match SDLC's deployed canonical files byte-for-byte and mode-for-mode.
- The propagation command proves each replaced file is a historical canonical version before overwriting it. Any unproven divergence is a refusal, not a reason to force the copy.
- Consumer tests exercise owned-orphan recovery, safe landing after unrelated ticket-only main movement, withdrawal receipt cleanup, and malformed `opens:` health reporting using BioMCP's adopted files.
- BioMCP has no executable `sdlc/scripts/deploy`, so the optional deployment extension remains a quiet no-op.
- After landing, BioMCP no longer appears in `factory doctor`'s project-script drift report. Other consumers may still keep doctor nonzero until their own adoption tickets land.

## Hard choice settled here

Copy the canonical files rather than reimplementing or selectively backporting incident fixes. A project-specific lifecycle need belongs behind a canonical extension or in `sdlc/scripts`, not in a private copy of one of the five lifecycle scripts.

## Boundary

- No BioMCP runtime, biomedical behavior, dependency, test fixture, or release change.
- No project-specific deploy hook.
- No change to SDLC's canonical scripts in this ticket.
