# Optimize — 369 Score variant-agent source APIs

## Starting Baseline

Exploit contract numbers were reproduced before optimization: 21 candidate sources, 12 good BioMCP proxy candidates, 8 possible-but-gated, 1 reject/default-exclude, 6 boundary rows, and 4 follow-up recommendations.

Primary optimization metric: end-to-end experiment benchmark-suite wall time for the three spike scripts, while preserving probe status/shape and matrix contract counts.

| Component | Baseline result | Baseline wall time |
|---|---:|---:|
| Current BioMCP CLI probes | 11 probes | 9.54 s |
| External API probes | 18 probes | 8.22 s |
| Source feasibility matrix synthesis | 21 candidates | 0.04 s |
| End-to-end suite | all counts reproduced | 17.80 s |

## Optimization Passes

| Pass | Hotspot | Approach | Before | After | Result |
|---|---|---|---:|---:|---|
| 1 | `probe_external_apis.py::request_probe` serial HTTP calls; per-request sum 8153.05 ms | Bounded parallel external probes with `ThreadPoolExecutor(max_workers=6)`, preserving result order | 17.80 s suite / 8.22 s external | 11.44 s suite / 2.43 s external | Committed; suite 35.7% faster |
| 2 | `probe_current_biomcp.py::run_probe` serial subprocess/network probes; per-command sum 8927.88 ms | Bounded parallel BioMCP CLI probes with `ThreadPoolExecutor(max_workers=4)`, preserving result order | 11.44 s suite / 8.97 s current | 6.89 s suite / 5.47 s current | Committed; suite 39.8% faster vs pass 1 |
| 3 | Residual 4-worker queue delayed long article probes | Raised current-probe fan-out from 4 to 8 workers | 6.89 s suite / 5.47 s current | 4.27 s suite / 3.16 s current | Committed; suite 38.0% faster vs pass 2 |
| 4 | Residual external-probe fan-out after pass 3 | Tried raising external fan-out from 6 to 12 workers | 4.27 s suite / 1.08 s external | 5.64 s suite / 0.80 s external | Reverted; primary suite metric regressed due current-probe network jitter |

Validation guardrail: final live reruns hit unauthenticated Semantic Scholar HTTP 429 after repeated benchmark executions. The probe harness now sends `x-api-key` for the Semantic Scholar probe when `S2_API_KEY` is present, matching BioMCP's optional authenticated-source behavior. This restored the regression-control status to HTTP 200.

## Final Numbers

Final measured suite after committed optimizations and the Semantic Scholar auth guardrail:

| Component | Final result | Final wall time |
|---|---:|---:|
| Current BioMCP CLI probes | 11 probes; expected return codes preserved | 4.74 s |
| External API probes | 18 probes; expected statuses preserved (`ensembl_vep` expected HTTP 400) | 1.60 s |
| Source feasibility matrix synthesis | 21 candidates | 0.04 s |
| End-to-end suite | contract counts and status/shape preserved | 6.38 s |

Validation run:
- Contract/status jq checks passed.
- `~/workspace/scripts/lint-planning.sh biomcp` passed (`all clean`).
- `cargo test --workspace --all-targets` passed: 1965 unit tests plus integration tests, 0 failures.

## Total Improvement

| Metric | Baseline | Final | Improvement |
|---|---:|---:|---:|
| End-to-end benchmark-suite wall time | 17.80 s | 6.38 s | 64.2% faster |
| Current BioMCP probe component wall time | 9.54 s | 4.74 s | 50.3% faster |
| External API probe component wall time | 8.22 s | 1.60 s | 80.5% faster |
| Matrix synthesis wall time | 0.04 s | 0.04 s | unchanged |
| Candidate sources | 21 | 21 | preserved |
| Classification counts | 12 / 8 / 1 | 12 / 8 / 1 | preserved |
| Follow-up recommendations | 4 | 4 | preserved |

The regression-control benchmark matches the explore/exploit contract on source/status shape and beats the starting end-to-end benchmark wall time. Individual public-network probe latencies still fluctuate, so latency-only differences remain noise-carved out as in the exploit report.

## Convergence

Optimization stopped after pass 4. The fourth pass improved the isolated external component (1.08 s → 0.80 s) but regressed the primary end-to-end suite wall time (4.27 s → 5.64 s), so it was reverted. That produced less than 5% improvement on the primary metric after three committed improvement passes, satisfying the convergence rule.

The remaining critical path is dominated by upstream public-network latency in a few article/current BioMCP probes. The benchmark harness is already close to the slowest-request lower bound for the current fixed probe set; pushing more concurrency increases jitter/rate-limit risk rather than improving the primary metric reliably.

## Remaining Opportunities

- Use fixture-backed/offline regression tests for stable timing; live public-network timing is inherently noisy.
- Split benchmark metrics into correctness/status control and optional latency telemetry so upstream slowness cannot obscure local harness performance.
- Add per-service rate-limit-aware scheduling to keep concurrency high without triggering shared-pool throttles.
- Reuse a single long-lived BioMCP process or MCP session for current-surface probes if future tickets need lower process-startup overhead.
- Keep production connector/API implementation out of this ticket; the optimized code is limited to experiment scripts and benchmark stability.
