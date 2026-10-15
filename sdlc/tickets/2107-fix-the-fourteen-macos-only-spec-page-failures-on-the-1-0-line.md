---
flow: build
priority: 2
deps: [2098]
---

# 2107: Fix the fourteen macOS-only spec page failures on the 1.0 line

Status: OPEN.

Milestone: 1.0

## Evidence

- 2098's acceptance run on M5: 14 spec pages fail on macOS while the fixture orchestration itself works. Standalone reproduction with the fixture serving (spec/entity/disease.md: 14 passed/5 failed) shows page-level findings — e.g. CAD candidates print "(no label in the search response)" — plus a page-block Bash 3.2 parse error and an mcp.md typed-schemas block failure. These are product/page findings surfaced by the now-working M5 gates, not fixture-orchestration defects.

## Work

- Reproduce each failing page on M5, separate macOS-only behavior (Bash 3.2 blocks, path handling) from real line regressions (missing labels), fix or ticket each with its own evidence.

## Acceptance

1. `make spec` on M5 fails no page, or every remaining macOS-only page carries a recorded refusal reason.
