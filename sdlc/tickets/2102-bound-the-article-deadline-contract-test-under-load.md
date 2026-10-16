---
flow: build
priority: 2
deps: []
---

# 2102: Bound the article deadline contract test under load

Status: COMPLETE.

Milestone: 0.9.3

Reviews: revision 2a84466, accept

## Evidence

- Test: raw_and_typed_article_query_validation_converges_before_provider_work (tests/rmcp_client_contract.rs, ~line 1146).
- Under two concurrent jobs it exceeded its 60s deadline (60.183s observed), killing full-suite runs. The same failure reproduces on unmodified main (base f8bf1d7) in a Yellow job, and ticket 2074's lane documented the identical behavior the same day.
- It passes isolated and on CI. Per the PM's 2026-10-15 doctrine, a timing test failing only with two jobs running is a test defect: bound it or shed its load sensitivity, do not accept the cost.

## Work

- Raise or remove the deadline for this contract test, or split the converging pair so neither depends on provider latency under contention. Whatever shape lands, the test must pass with two jobs running.

## Acceptance

1. The test passes in a full suite while another Yellow job runs concurrently, twice.

## Severity update 2026-10-15 (from 2073's lane)

- `raw_and_typed_article_query_validation_converges_before_provider_work` now fails single-job on unmodified main `526902d36` (60.198s > 60s), not only under two-job contention. The acceptance bar (passes with two jobs running, twice) should also require a single-job quiet-host pass.

