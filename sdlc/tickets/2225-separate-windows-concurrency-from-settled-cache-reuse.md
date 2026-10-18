# 2225 — Resolve the Windows parallel warm-cache failure

Status: OPEN.

Milestone: 0.9.4

Reviews: revision 88393e5, accept

## Outcome

Determine whether the Windows warm-cache failure reflects a product defect or an invalid test assumption, including comparison with released v0.9.3. Fix the demonstrated cause while preserving successful concurrent CLI and MCP responses, fail-open caching, and exact settled byte accounting.

## Changes

`tests/windows_stdio_contract.rs`, `.github/workflows/ci.yml`, `tests/test_ci_workflow_contract.py`, `CHANGELOG.md` and this ticket. Scope is a bounded Windows diagnostic and demonstrated test repair; any product-code defect returns for scope review before implementation.

## Keeps

Do not change production locking or demand every contended cache write persists. Keep real Windows concurrency, cache-clear and accounting assertions. Retain explicit manual-only Windows workflow dispatch and unchanged ordinary release behavior. No automatic triggers or general diagnostic framework.

## Proof

Manual Windows run39166219806 at add602786ade203d31f58331fd8a2cc039470bd3 failed in windows_parallel_cli_and_mcp_survive_shared_cache_clear_and_keep_byte_totals at line431: provider requests5 versus expected4 after successful parallel cold responses. Previous run39159496454 passed this case. The failed run did not reach the renamed body-limit selector. Current inspection suggests nonblocking HTTP byte-total locking can legitimately skip a concurrent cache write, but the actual missed entry and lock cause remain unproven. Compare released v0.9.3 on the same Windows fixture and collect enough bounded diagnostics to classify the failure before fixing it. If only the assumption is wrong, establish the required entries sequentially after the concurrent phase, then assert subsequent CLI and MCP reuse without extra provider requests. Reuse the same scoped manual run for final2219 selector proof when possible.

## Progress

- 2026-10-18 started: Restated outcome is to distinguish a real Windows cache-reuse defect from permitted fail-open cache writes, compare the released version, and repair only the demonstrated cause. One repair lane replaces the blocked2219 development phase; other landings continue.
- 2026-10-18 started
- 2026-10-18 next: Component accepted bb799c1b3de8a09b8d59c536a1fbdd978e0d2e20 after concurrency-barrier repair and qualified-main integration. Manual Windows39167643933 verified exacthead bb799c1b3de8a09b8d59c536a1fbdd978e0d2e20. Final acceptance waits released/current real-lock, originalwarm-oracle, settledreuse/accounting and renamedselector proof. No product defect or runtime pass claimed.
