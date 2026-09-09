# ClinGen CAR rate limiting degrades batch identity verification

Found during a full CLI verification run (2026-09-09) on main `9f6cbc0e`:
`make verify` failed 3 of 58 live assertions (55 passed, 19 skipped), all in
`spec/entity/variant-articles-live.md` — the G5 v2 identity canary (two
assertions) and the provider-specific strict query provenance canary.

Evidence it is provider-side, not a code defect: the 7-item G5 identity panel
loses ClinGen CAR confirmation on the last 3 items with one provider timeout
on items 3 and 4, while every item resolves alone (BRCA1 confirms CA001621,
PALB2 confirms CA168760). The offline captured-corpus spec lane passes on the
same commit.

Mechanism: ClinGen CAR calls use the advisory `SourceContext::retry` only
(`src/sources/clingen_allele_registry.rs:112,157,180`); the shared
`retry_send` 429-with-backoff machinery at `src/sources/mod.rs:1364` is not
applied to CAR; item concurrency 2 and provider concurrency 10
(`src/entities/article/variant_search.rs:193-194`) burst the tail items into
a live rate limiter.

Options for a future ticket: apply `retry_send` to ClinGen CAR (preferred; it
also fixes user-facing rate-limit resilience), or have the live canary merge
readiness across a delayed second run while still asserting work-allocation
consistency on the first full run. No design decided here. Per
`sdlc/planning/verify-lane.md`, live failures are not gate failures; this
note records the observation for triage.
