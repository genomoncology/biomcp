# 2225 — Resolve the Windows parallel warm-cache failure

Status: OPEN.

Milestone: 0.9.4

Reviews: revision 88393e5, accept

Reviews: revision 100e4d3, accept

## Outcome

Determine whether the Windows warm-cache failure reflects a product defect or an invalid test assumption, including comparison with released v0.9.3. Fix the demonstrated cause while preserving successful concurrent CLI and MCP responses, fail-open caching, and exact settled byte accounting.

## Changes

`tests/windows_stdio_contract.rs`, `.github/workflows/ci.yml`, `tests/test_ci_workflow_contract.py`, `CHANGELOG.md`, `sdlc/tickets/2227-log-routine-http-cache-lock-contention-at-debug.md` and this ticket. Scope is a bounded Windows diagnostic and demonstrated test repair; any product-code defect returns for scope review before implementation.

## Keeps

Do not change production locking or demand every contended cache write persists. Keep real Windows concurrency, cache-clear and accounting assertions. Retain explicit manual-only Windows workflow dispatch and unchanged ordinary release behavior. No automatic triggers or general diagnostic framework.

## Proof

Manual Windows run39166219806 at add602786ade203d31f58331fd8a2cc039470bd3 failed in windows_parallel_cli_and_mcp_survive_shared_cache_clear_and_keep_byte_totals at line431: provider requests5 versus expected4 after successful parallel cold responses. Previous run39159496454 passed this case. The failed run did not reach the renamed body-limit selector. Current inspection suggests nonblocking HTTP byte-total locking can legitimately skip a concurrent cache write, but the actual missed entry and lock cause remain unproven. Compare released v0.9.3 on the same Windows fixture and collect enough bounded diagnostics to classify the failure before fixing it. If only the assumption is wrong, establish the required entries sequentially after the concurrent phase, then assert subsequent CLI and MCP reuse without extra provider requests. Reuse the same scoped manual run for final2219 selector proof when possible.

## Progress

- 2026-10-18 started: Restated outcome is to distinguish a real Windows cache-reuse defect from permitted fail-open cache writes, compare the released version, and repair only the demonstrated cause. One repair lane replaces the blocked2219 development phase; other landings continue.
- 2026-10-18 started
- 2026-10-18 next: Component accepted bb799c1b3de8a09b8d59c536a1fbdd978e0d2e20 after concurrency-barrier repair and qualified-main integration. Manual Windows39167643933 verified exacthead bb799c1b3de8a09b8d59c536a1fbdd978e0d2e20. Final acceptance waits released/current real-lock, originalwarm-oracle, settledreuse/accounting and renamedselector proof. No product defect or runtime pass claimed.

## Actual Windows result

Manual run39167643933 succeeded at exactbb799c1b3de8a09b8d59c536a1fbdd978e0d2e20. Official0.9.3 archiveSHA25629dc53bf037c9d210280b6317e6155b84cd82f2c357a94046b9d5053b1c9a881; executableSHA2562e7a82c7e1a3c7bd056cb22d858c477f6f7f285d5f2dcacdc54324f4e5710d9b and banner0.9.3 verified. Released and current both reproduced the original warm oracle with CLI0misses and MCP1miss. In both, deliberate byte-total lock contention gave a correct response, absent BRAF_LOCK entry and actual write warningWouldBlock. Sequential refill then warm reuse, concurrent clearing and settled accounting passed. Released case1passed1.38s; currentstdio2passed18.74s. The renamed body-limit case actually selected1test and passed0.01s; lock and construction deadline checks also passed. The separate managed-boundary compile step was skipped in this scoped run and is not claimed.

This supports a test-assumption repair, not a persistent cache-reuse defect. Preserve fail-open behavior. PM approved landing this ticket solo with the imported2219selector/guard, then withdrawing2219 as superseded with its proof retained. The visible routine-contention WARN is filed separately as priority3 ticket2227. No extra Windows run is required for record-only integration changes.
