---
flow: build
priority: 5
---

# Post-write cache metadata failure has an untested public outcome

## Goal

Make the fail-closed outcome after a successful CACache write explicit and
deterministically tested through both the cache manager and the shared cached
HTTP client.

At commit `34daa72b`, `SizeAwareCacheManager::put` delegates the write and then
reads CACache metadata to locate and harden the exact content blob. Missing
metadata or a metadata-read error is propagated. This can turn a successful
upstream cacheable response into a provider-request error after the response
has already been persisted. Before ticket 1167, those failures were warnings
and the upstream response was returned.

Ticket 1167's completion record deliberately describes the new behavior as
fail-closed because BioMCP cannot otherwise prove that the written content path
is private. Its accepted review considered post-write attribution, but the
original acceptance criteria did not state this public outcome and no focused
regression exercises either failure branch.

## Result

The fail-closed boundary is now explicit at both layers. After CACache returns
from a delegated write, missing or unreadable metadata is mapped to the stable
BioMCP-owned context `cache security finalization failed after successful put`.
The cached response is not returned, and the public middleware error contains
neither the cache root, cache key, nor upstream response body.

A test-only manager factory keeps the production middleware order, cache
options, and `SizeAwareCacheManager` while allowing exact manager `get` and
post-delegation `put` observation. Deterministic loopback coverage proves an
initial cacheable `200`, stale conditional `304`, and stale replacement `200`
each perform one target upstream request and one target write before failing.
It also proves a fresh hit performs no upstream request or write, while a
request-level `CacheMode::NoStore` bypasses manager `get` and `put` and returns
its one upstream response. Revalidation failures are armed only after the
stale seed succeeds.

Focused validation passed 23 cache-manager tests and 10 provider-network and
cache-construction tests. Focused Clippy with warnings denied, formatting,
diff whitespace validation, and the repository quality ratchet passed. The
ratchet retains the exact 1,884-line `src/sources/mod.rs` baseline; no tracked
file was added to the shipped package.

## Done, observably

- A deterministic manager test first proves that the delegated CACache write
  completed, then removes its metadata and proves that `put` returns an error
  rather than an apparently successful cache result. A separate case injects
  a metadata-read error and requires a stable BioMCP-owned internal
  classification or context without pinning platform-specific I/O text.
- Shared cached-client coverage uses a test-only injection point that preserves
  the production middleware ordering, cache options, and
  `SizeAwareCacheManager` implementation. It proves these exact branches:
  - An initial cacheable `200` miss makes one upstream request and one manager
    `put` attempt; the caller receives an error and no response body.
  - A stale cached entry revalidated by conditional `304` sends the validator,
    makes one upstream request and one rewrite attempt, and returns an error
    rather than cached content.
  - A stale cached entry revalidated by `200` makes one upstream request and
    one rewrite attempt, and returns an error rather than the new response.
  - A fresh cache hit makes no upstream request and no `put` call, and returns
    the cached response successfully.
  - A request carrying `CacheMode::NoStore` makes one upstream request, makes
    no manager `get` or `put` call, and returns the upstream response.
- Revalidation setup uses a one-shot armed failure so seeding the stale entry
  succeeds and only the target rewrite fails. Tests do not depend on timing,
  filesystem permissions unavailable to the test user, or a provider network.
- Public cached-client errors do not expose the cache path, cache key, or
  upstream response body.
- The completion record states plainly that cache persistence hardening can
  fail an otherwise successful upstream request. If implementation evidence
  shows that `http-cache` does not propagate one of the named paths, the ticket
  returns to design review instead of asserting a false contract.

## Boundary

This ticket pins the existing fail-closed security boundary. It does not return
potentially unsecured cached content, change cache layout or eviction, add
provider retries, or prune `.biomcp-key-locks`. Those lock files are deliberately
retained, shared across operations, and bounded to 256 SHA-1 shards per cache
root; they are not one file per distinct cache key.

## Review

- Design review: accepted after remediation pinned the exact cached-client
  branches, required production-shaped manager injection, distinguished
  request `CacheMode::NoStore`, and bounded internal and public error context.
- Code review: the first review rejected compaction that retained the legacy
  migration tests but dropped their decisive migrated-sentinel absence checks.
  Both assertions are restored at the exact source-line baseline; focused
  remediation review is pending.
