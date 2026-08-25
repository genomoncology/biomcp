---
flow: build
priority: 30
waits-on: ["botassembly/sdlc/0167"]
---
# BioMCP adopts transient fetch recovery

SDLC 0167 makes canonical `tasks` retry one failed fetch and preserve a useful bounded diagnostic after two failures. BioMCP carries a copied lifecycle set, so its deployed checkout must adopt that canonical behavior before normal dispatch resumes.

## What done looks like, observably

- All five `sdlc/project` files match SDLC's deployed canonical bytes and executable modes.
- Propagation proves every replaced file is a historical canonical version and refuses an unproven customization.
- The propagation command uses a temporary registry containing only this attempt's BioMCP worktree. It never uses the live registry or modifies another checkout.
- Consumer tests exercise first-fetch-fails/second-fetch-succeeds and double-fetch-fails diagnostics through the adopted `tasks` script without contacting a real remote.
- BioMCP's domain tests and project-owned gates retain their current behavior.
- Doctor reports no lifecycle-script drift for BioMCP after landing.

## Boundary

- No BioMCP command, adapter, domain, fixture, specification, or provider behavior changes.
- No SDLC canonical file changes.
- No other registered checkout is modified.
