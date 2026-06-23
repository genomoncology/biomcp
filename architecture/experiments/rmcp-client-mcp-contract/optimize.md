# Optimize — rmcp Rust client MCP contract migration

## Starting Baseline

Starting contract numbers came from `.march/exploit.md` and were reproduced before any optimization.

Exploit baseline:

| Metric | Baseline |
|---|---:|
| Full focused rmcp contract run | 3.812 s |
| Tests | 6 passed, 0 failed |
| Stdio core regression control | 0.022 s |
| HTTP core regression control | 0.054 s |
| Stdio chart contract | 0.058 s |
| HTTP chart contract | 0.085 s |
| Stdio full contract | 3.812 s |
| HTTP full contract | 0.865 s |
| Mismatch count | 0 |

Reproduced baseline with `cargo nextest run --test rmcp_client_contract`:

| Metric | Reproduced baseline |
|---|---:|
| Full focused rmcp contract run | 3.818 s |
| Tests | 6 passed, 0 failed |
| Stdio core regression control | 0.029 s |
| HTTP core regression control | 0.059 s |
| Stdio chart contract | 0.060 s |
| HTTP chart contract | 0.085 s |
| Stdio full contract | 3.818 s |
| HTTP full contract | 0.853 s |

Primary metric: full focused rmcp contract runtime.

## Optimization Passes

### Pass 1 — bound stdio full-contract teardown

- Hotspot: `rmcp_child_process_client_verifies_stdio_full_contract`, final `client.cancel().await?` in `tests/rmcp_client_contract.rs`.
- Profile result: manual phase timing showed the contract assertions took about 0.77 s, while `client.cancel()` took about 3.00 s.
- Why slow: rmcp's child-process transport waits up to its 3 s graceful-shutdown timeout for the child process to exit. That cleanup wait was not contract work.
- Approach: capture the stdio child PID before handing the transport to rmcp, send `TERM` after assertions pass, then await `client.cancel()` so cleanup remains explicit and nextest reports no leak.

| Metric | Before | After |
|---|---:|---:|
| Full focused rmcp contract run | 3.818 s | 1.329 s |
| Stdio full contract | 3.818 s | 1.271 s |
| HTTP full contract | 0.853 s | 1.328 s |
| Stdio core regression control | 0.029 s | 0.039 s |
| HTTP core regression control | 0.059 s | 0.067 s |

Decision: committed (`0615b857`). Primary metric improved 65.2%.

### Pass 2 — stub optional MedlinePlus enrichment locally

- Hotspot: `assert_read_only_and_policy_calls`, specifically `biomcp discover BRCA1`.
- Profile result: HTTP full-contract timing showed `assert_read_only_and_policy_calls` cost about 936 ms, with `discover` alone about 901 ms.
- Why slow: `discover` fans out to OLS4, UMLS, and MedlinePlus. OLS4 was stubbed and UMLS was disabled, but optional MedlinePlus enrichment still pointed at its default external endpoint. Its result is not needed for the MCP contract assertion.
- Approach: set `BIOMCP_MEDLINEPLUS_BASE` to the existing local stub URL in both full-contract tests, making optional enrichment fail fast locally while OLS4 still supplies the BRCA1 result under test.

| Metric | Before | After |
|---|---:|---:|
| Full focused rmcp contract run | 1.329 s | 0.912 s |
| Stdio full contract | 1.271 s | 0.880 s |
| HTTP full contract | 1.328 s | 0.911 s |
| Stdio core regression control | 0.039 s | 0.028 s |
| HTTP core regression control | 0.067 s | 0.060 s |

Decision: committed (`b9f580e8`). Primary metric improved 31.4%.

### Pass 3 — dedicated MedlinePlus XML stub

- Hotspot: remaining `discover` latency inside `assert_read_only_and_policy_calls`.
- Profile result: after pass 2, the `discover` call still measured about 794 ms in a focused HTTP full-contract run.
- Why tried: if the remaining time came from pointing MedlinePlus at a JSON-shaped OLS stub, a minimal XML stub could make the optional enrichment path cheaper.
- Approach: add a dedicated local MedlinePlus stub returning a minimal XML response and point `BIOMCP_MEDLINEPLUS_BASE` to it.

| Metric | Before | After |
|---|---:|---:|
| Full focused rmcp contract run | 0.912 s | 0.954 s |
| Stdio full contract | 0.880 s | 0.903 s |
| HTTP full contract | 0.911 s | 0.954 s |
| Stdio core regression control | 0.028 s | 0.028 s |
| HTTP core regression control | 0.060 s | 0.058 s |

Decision: reverted. It regressed the primary metric by about 4.6% and added complexity without payoff.

## Final Numbers

Final benchmark after reverting pass 3:

```bash
cargo nextest run --test rmcp_client_contract
```

| Metric | Final |
|---|---:|
| Full focused rmcp contract run | 0.933 s |
| Tests | 6 passed, 0 failed |
| Stdio core regression control | 0.029 s |
| HTTP core regression control | 0.059 s |
| Stdio chart contract | 0.059 s |
| HTTP chart contract | 0.086 s |
| Stdio full contract | 0.891 s |
| HTTP full contract | 0.933 s |
| Mismatch/correctness failures | 0 |

Validation after committed optimizations:

- `make lint`: passed.
- `make test`: passed.
- `make spec`: passed.

## Total Improvement

Baseline below uses the reproduced pre-optimization numbers from this worktree.

| Metric | Baseline | Final | Change |
|---|---:|---:|---:|
| Full focused rmcp contract run | 3.818 s | 0.933 s | 75.6% faster |
| Stdio core regression control | 0.029 s | 0.029 s | unchanged |
| HTTP core regression control | 0.059 s | 0.059 s | unchanged |
| Stdio chart contract | 0.060 s | 0.059 s | 1.7% faster |
| HTTP chart contract | 0.085 s | 0.086 s | 1.2% slower |
| Stdio full contract | 3.818 s | 0.891 s | 76.7% faster |
| HTTP full contract | 0.853 s | 0.933 s | 9.4% slower |
| Correctness failures | 0 | 0 | unchanged |

The primary metric improved substantially, and the explore regression controls still match or beat the explore baselines (`stdio core 0.023 s`, `HTTP core 0.276 s`, mismatch count 0).

## Convergence

Stopped after three passes because pass 3 did not improve the primary metric and was reverted. The remaining hotspot is the `discover` command's enrichment path inside the full MCP contract tests. Further material gains likely need a broader test seam or command-mode change for discover enrichment, not a small isolated optimization.

## Remaining Opportunities

- Add an explicit test-only or documented no-enrichment mode for `discover`, if future contract tests need to assert routing without optional enrichment fan-out.
- Split full-contract transport parity so only one transport performs the expensive `discover` coverage and the other verifies transport-specific behavior. That would be an intentional coverage-shape change, not a micro-optimization.
- Investigate why `discover` still spends roughly 800 ms even with optional sources pointed locally. This may involve rate limiting, timeout coordination, or request middleware behavior.
- Upstream rmcp could expose a faster child-process shutdown path or a configurable graceful-shutdown timeout, removing the need to terminate the child PID in the test harness.
