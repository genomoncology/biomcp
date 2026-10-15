---
flow: build
priority: 2
deps: []
---

# 2102: Bound the article deadline contract test under load

Status: OPEN.

Milestone: 0.9.3

## Evidence

- Test: raw_and_typed_article_query_validation_converges_before_provider_work (tests/rmcp_client_contract.rs, ~line 1146).
- Under two concurrent jobs it exceeded its 60s deadline (60.183s observed), killing full-suite runs. The same failure reproduces on unmodified main (base f8bf1d7) in a Yellow job, and ticket 2074's lane documented the identical behavior the same day.
- It passes isolated and on CI. Per the PM's 2026-10-15 doctrine, a timing test failing only with two jobs running is a test defect: bound it or shed its load sensitivity, do not accept the cost.

## Work

- Raise or remove the deadline for this contract test, or split the converging pair so neither depends on provider latency under contention. Whatever shape lands, the test must pass with two jobs running.

## Acceptance

1. The test passes in a full suite while another Yellow job runs concurrently, twice.
