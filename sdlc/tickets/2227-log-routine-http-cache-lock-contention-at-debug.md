---
priority: 3
---

# 2227 — Log routine HTTP cache lock contention at debug

Status: OPEN.

Milestone: 0.9.4

Reviews: revision 446e256, accept

## Outcome

Ordinary nonblocking HTTP cache-write lock contention returns the provider answer and logs at debug level. Preserve visible warnings for genuine cache I/O failures.

Depends on: 2225

- Changes: `src/cache/http.rs`, `tests/test_http_cache_fail_soft.py`, `tests/windows_stdio_contract.rs`, `CHANGELOG.md`, `sdlc/tickets/2228-document-python-job-bootstrap-without-packaging.md` and this ticket. Keep the change in shared error classification or logging rather than adding provider-specific handling.

## Evidence

Windows comparison39167643933 on the verified official0.9.3 executable and currentcandidatebb799c1b3 reproduces a successful provider answer with an intentionally skipped write under the held byte-total lock. Both emit `local HTTP cache write failed; continuing` at WARN with `io_kind=Some(WouldBlock)`. Routine contention is expected fail-open behavior. The PM requested this small priority3 follow-up on2026-10-18, consistent with2194's debug-level contention policy.

## Keeps

Do not block requests or require every contended cache write to persist. Do not suppress warnings for real filesystem errors or change cache retention, accounting, lock semantics or provider answers.

## Proof

Use the real byte-total lock to reproduce a successful response and skipped cache write. At normal warning verbosity, routine WouldBlock contention emits no WARN; debug verbosity retains the contention explanation. A real non-contention I/O failure still warns and leaves the provider response intact. Reuse existing fixture evidence and run affected checks rather than another broad platform matrix.

## Progress

- 2026-10-18 started

## Development brief and reproduction

Restated outcome: classify expected nonblocking cache contention before the shared once-per-process warning guard, log it at debug, and preserve genuine I/O warnings plus provider answers. The current shared warn_local_http_cache helper consumes WARNED before classification, so ordinary contention both emits WARN and can suppress a later genuine fault. HTTP put/delete share nonblocking operation, key and byte-total lock paths; setup contention already logs at debug through the2194policy. No manager/lock/deadline/provider behavior changes.

Actual Windows39167643933 on currentcandidatebb799c1b3 and official0.9.3 ran the real held byte-total lock case. Command: `cargo test --locked --no-default-features --test windows_stdio_contract -- --nocapture`. Both returned the correct BRAF answer, showed BRAF_LOCK absent, and logged `WARN local HTTP cache write failed; continuing` with operationwrite/categorylocal_io/io_kindSome(WouldBlock). The same shared logger is unchanged in current qualified2a711534e4f4d588f1b2be6f48f3f3cd16cd864a, whose fullmain job2156/1792360049283-18457 passed221.910s.

Extend the existing Python fail-soft fixture with real held byte-total lock controls at warning/debug verbosity: correct provider answer, skipped index entry, no WARN for ordinary contention, and a debug explanation only at debug. Preserve the real non-contention filesystem-failure warning control and subsequent refetch/reuse after unlock. Keep routine contention outside the global warning allowance. Adjust the existing Windows diagnostic to distinguish the explicitly configured legacy0.9.3comparison from current logging; retain successful answer/contention/accounting semantics. Shared logging-only behavior needs no extra broad Windows dispatch.

Write the failing desired-verbosity assertions before the production change, then run affected red/green checks through one bounded build job using a frozen qualified baseline and one ordinary candidate build. Preserve exact commands/results and run format/lint/quality checks; no new harness, live provider call or WIP push.

## Related instruction prevention

The separate priority3 ticket2228 records the repeated Python setup/packaging and Ruff-location mistakes seen in multiple lanes. It changes instructions only and does not broaden this logging fix.
