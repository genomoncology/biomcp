---
flow: build
priority: 19
waits-on: ["botassembly/sdlc/0164"]
---
# BioMCP adopts activation-aware project settlement

Factory 0090 adds the durable consumer behavior for a landing whose registered checkout still needs activation. SDLC 0164 then changes canonical `success` to produce that result and support activation-only reconciliation. BioMCP keeps its domain behavior below the lifecycle boundary, so all five copied project scripts can adopt the deployed canonical contract unchanged.

## What done looks like, observably

- All five `sdlc/project` files match SDLC's deployed canonical bytes and executable modes.
- Propagation proves every replaced file is a historical canonical version and refuses an unproven customization.
- The proof uses a temporary project registry containing only this attempt's BioMCP worktree. It never invokes zero-argument propagation against the live registry and never modifies another registered checkout.
- Lifecycle tests exercise the activation-pending result and activation-only invocation with the adopted script. BioMCP's domain tests and project-owned gates retain their current behavior.
- Doctor reports no lifecycle-script drift for BioMCP after landing.

## Hard choice settled here

Adopt the complete canonical set mechanically rather than selectively merging `success`. BioMCP-specific behavior belongs in its project-owned gates and hooks, not a private lifecycle fork.

## Boundary

- No BioMCP command, adapter, domain, fixture, specification, or provider behavior change.
- No SDLC canonical-script change.
- No Factory, Botassembly, Deck, or SDLC registered checkout is modified by this ticket's propagation command.
