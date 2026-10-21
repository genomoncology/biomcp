---
flow: build
priority: 10
deps: [1167, 1173]
---

# Return variant literature within one invocation-wide work deadline

## Outcome

Variant-literature retrieval now shares one monotonic 60-second provider-work
deadline across resolution, route searches, enrichment, pagination, and
publication. Work admitted before the deadline is retained; work that cannot be
started or completed is represented by the typed route ledger without
inventing completeness or totals. Cache maintenance and per-key write
coordination no longer deadlock a request, and cancellation cannot leak a
shared operation lease.

The implementation preserves bounded full-width ranking, source and alias
provenance, resolution-first hard failures, and the existing public metadata
shape. Large article and disease futures use dedicated boxed dispatch arms to
stay within the process stack contract. The deterministic Semantic Scholar
fixture now implements the real batch-enrichment POST and returns one
ID-aligned nullable result per requested paper, so a healthy fixture no longer
creates a false provider failure.

## Review

Independent design and code review rejected intermediate revisions for missing
per-key deadline checks, incomplete plan reconciliation, a cache-lock upgrade
deadlock, cancellation-unsafe lease acquisition, stack overflow on large
dispatch futures, and an omitted test-only HTTP-client inventory entry. Each
finding received a focused red-green remediation. The final review accepted
`567f8cbdae5d7f3c91779860ace8b0690db533cb` with no findings. It confirmed
truthful complete/total semantics, authoritative terminal counts and reasons,
preserved route-specific detail, unchanged prior security behavior, and an
exact 1,300-file source package.

## Verification

The final reviewed revision, based on BioMCP main `e8f02aa8` and BioData
0.0.11 at `cfafc69d`, passed `make lint`, `make test`, `make spec`, and
`make full-feature-check` under Node 22. The routine test gate ran 3,321 Rust
tests with 3,321 passed and 30 skipped, then 912 Python tests with 912 passed
and 3 skipped; strict documentation also passed. Every routine and static
mustmatch suite passed, including 103 variant cases with one skipped. The six
AlphaGenome tests, all-feature Clippy, optimized release build, and PNG, SVG,
and terminal artifact smoke all passed.

## Boundary

This ticket owns the invocation-wide variant-literature work budget and its
truthful settlement. It does not change public input schemas, provider retry
policy, cache layout, or the ranking optimization reserved for ticket 1164.
