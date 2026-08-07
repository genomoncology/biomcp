---
flow: build
priority: 7
---
# Harden post-migration spec runner and CLI surface ratchets

The migration made the routine spec lane much healthier, but review found many assertion-strength gaps where contracts can stay green while the user-visible behavior or runner participation regresses. These should become automated spec/lint/contract pins rather than FAQ watchpoints.

Completed under March on 2026-06-09, as March ticket 401. Imported as history when BioMCP
moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/401-harden-post-migration-spec-runner-and-cli-surface-ratchets
