---
base: 0e96a4e3
head: 8c37f045
---

Stopped the cache mode leaking across tests. Ian filed the issue
from GitHub #286 and named the cause: `env_cache_mode()` cached the
first `BIOMCP_CACHE_MODE` value in a process-wide OnceLock, so tests
setting the variable to `off` left the bypass latched for the rest
of the binary — order-dependent failures on 0.9.0 (8 failed
single-threaded, each passing alone).

The fix is the issue's option 2: release behavior unchanged (one
OnceLock read, no new env read or branch — the seam is `#[cfg(test)]`
and compiles out), and tests get a scoped guard
(`test_cache_mode::off()` / `::infinite()`) that sets a test-only
override slot and restores it on Drop. Every cache-mode reader now
goes through the one `current_cache_mode()` — including a third
reader the issue had not counted (fda_orphan's raw env string read)
— so no two readers can disagree about mode-change timing again.
The two setters the issue named no longer touch the env var, and a
third site (graph's infinite-mode test) had to convert with them
because a post-unification env set can no longer take effect
mid-process.

Tests: the guard's apply/restore/nesting/no-latch unit test
(serialized on the source_env key after review caught the shared
slot), and the issue's success criterion end to end — under the
guard the sidecar is not written; after the drop the same binary
writes it; a later read serves from it with no upstream requests.

Evidence: code review REJECT on two serious findings (the unserialized unit test
against the process-global slot — exactly the flake class this
ticket kills — and a stale line-count pin), both fixed; gate cycles
caught clippy (a Copy-clone and a held-for-Drop field needing the
repo's `dead-code reason:` comment form with its inventory entry)
and a rustfmt drift; green at 8c37f045 — lint, test, spec, stress,
zero failed lines. Close GitHub #286 when this reaches a release.
