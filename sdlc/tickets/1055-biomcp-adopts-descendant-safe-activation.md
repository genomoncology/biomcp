---
flow: build
priority: 30
waits-on: ["botassembly/sdlc/0168"]
---
# BioMCP adopts descendant-safe activation

SDLC 0168 corrects canonical activation so a clean current main that provably contains a stored landed tip can activate without rolling later work backward. BioMCP carries a copied lifecycle set and must adopt the deployed canonical correction.

## What done looks like

- All five `sdlc/project` files match the deployed SDLC canonical files byte-for-byte and mode-for-mode.
- Propagation proves every replaced file is a historical canonical version and uses a temporary one-project registry; it does not modify another registered checkout.
- Consumer tests reproduce a stored activation tip followed by a later main descendant and prove BioMCP's adopted `success` activates the current clean main and returns the original exact receipt.

## Boundary

- No BioMCP domain behavior, Factory queue behavior, or canonical SDLC change.
- No reset, clean, stash, force push, or manual queue settlement.
