---
flow: build
priority: 2
deps: [2041]
---

# 2099: Reconcile the 0.9 transport contracts on the 1.0 line

Status: OPEN.

Milestone: 1.0

## Evidence

- Filed 2026-10-15 from the 2041 merge: after main merges into the 1.0 line, 117 tests fail — concentrated in main's variant transport contracts (full-card JSON equality across CLI/MCP channels) and the stale-cache note tests. The 1.0 line's own transport tests passed at the base tip. These failures are the transport-contract face of the same deferral as 2097: main's 0.9 surfaces not yet reconciled with the 1.0 resolver structure.
- Changes: the merged tree's variant card projections and cache-note behavior, or the contract tests' pinned shapes, whichever the 1.0 structure says is authoritative.
- The failing test files and classes are listed verbatim in the 2041 build record.

## Acceptance

1. Every test failing at the 2041 landing passes, or is rewritten to the 1.0 line's authoritative projection with the change reviewed.
2. Full nextest over the prepared archive is green on the 1.0 line tip.

## Exclusions

No changes to 0.9 main behavior.
