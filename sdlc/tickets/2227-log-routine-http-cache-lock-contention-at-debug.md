---
priority: 3
---

# 2227 — Log routine HTTP cache lock contention at debug

Status: OPEN.

Milestone: 0.9.4

## Outcome

Ordinary nonblocking HTTP cache-write lock contention returns the provider answer and logs at debug level. Preserve visible warnings for genuine cache I/O failures.

Depends on: 2225

## Evidence

Windows comparison39167643933 on the verified official0.9.3 executable and currentcandidatebb799c1b3 reproduces a successful provider answer with an intentionally skipped write under the held byte-total lock. Both emit `local HTTP cache write failed; continuing` at WARN with `io_kind=Some(WouldBlock)`. Routine contention is expected fail-open behavior. The PM requested this small priority3 follow-up on2026-10-18, consistent with2194's debug-level contention policy.

## Changes

`src/cache/http.rs`, focused existing cache/CLI tests and `CHANGELOG.md`. Name the actual test paths before implementation. Keep the change in shared error classification or logging rather than adding provider-specific handling.

## Keeps

Do not block requests or require every contended cache write to persist. Do not suppress warnings for real filesystem errors or change cache retention, accounting, lock semantics or provider answers.

## Proof

Use the real byte-total lock to reproduce a successful response and skipped cache write. At normal warning verbosity, routine WouldBlock contention emits no WARN; debug verbosity retains the contention explanation. A real non-contention I/O failure still warns and leaves the provider response intact. Reuse existing fixture evidence and run affected checks rather than another broad platform matrix.
