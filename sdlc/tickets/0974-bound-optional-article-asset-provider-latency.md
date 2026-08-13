---
flow: build
priority: 9
deps: ["0973"]
---
# Bound optional article-asset provider latency

Article asset discovery currently waits for every joined provider observation.
When one optional Europe PMC endpoint hangs, useful PMC results that arrived in
seconds are withheld for roughly two minutes, and each paging continuation can
pay the same delay again.

Optional provider rungs must have a bounded overall attempt budget that covers
requests, retries, and retry sleeps. When that budget expires, asset discovery
returns the complete results already available from successful sources and
marks the unavailable coverage as degraded. A timeout from one optional source
must not turn a successful primary-source manifest into a command failure.

Normal cached paging must reuse the bounded discovery result sufficiently that
moving to the next asset page does not immediately repeat the same unavailable
optional-provider ladder. `--no-cache` remains an explicit request to avoid
managed reuse and may repeat provider work. Do not permanently negative-cache
a transient outage or describe an unavailable source as having no assets.

## Done when

- Deterministic delayed-provider fixtures prove one optional source cannot
  hold the command for multiple ordinary request-timeout cycles.
- Available assets are returned with explicit degraded or unavailable-source
  facts when an optional source exceeds its budget.
- A normal cached continuation does not immediately contact every failed
  optional source again, while expiry and `--no-cache` permit a fresh attempt.
- Successful, absent, failed, and timed-out sources remain distinguishable in
  JSON and Markdown coverage facts.
- Routine tests use local fixtures and virtual or tightly bounded test time;
  they do not add a slow wall-clock sleep or public-network dependency.

## Authorized test changes

Design may restate article-asset discovery, coverage, paging, and cache-reuse
assertions in `src/entities/article/assets.rs`, `src/cli/article/assets.rs`, and
`tests/unit/cli/article_assets.rs`. It may add focused local-provider fixtures
and test-only timeout controls needed to prove the overall budget without
slowing the routine suite.
