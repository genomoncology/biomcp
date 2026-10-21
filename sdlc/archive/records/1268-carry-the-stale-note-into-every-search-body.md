---
ticket: 1268
head: f86a548e
merge: 26538f27
---

# 1268 — the stale-cache claims

## What changed

GWAS keeps `NoStore` unconditionally, pinned twice: the runtime test `src/sources/gwas/tests/no_store.rs` reads `CacheMode::NoStore` through the public `RequestBuilder::extensions()` on reqwest-middleware 0.4.2 (the first attempt read reqwest's private `Request::extensions()` after `build()` and failed with E0624 in CI run 36619750222), and the text contract `tests/test_gwas_no_store_contract.py` stays as the structural belt. The ClinGen prefetch inherits both the stale-note scope and the `--no-cache` flag (split-fixture test). TLS trust failures are not retried on the middleware clients (`NoTrustFailureStrategy` on both client builders, with the chain-walk unit tests in `src/sources/trust_failure_tests/mod.rs`); the plain-send retry path `retry_middleware_send`, used by Enrichr and UniProt, now also stops at the first trust failure and surfaces the error (fixed 2026-09-30 on the 1279 branch, with an async test proving a single attempt); the tests live in `src/sources/trust_failure_tests/mod.rs`. The search-all end-to-end stale serve was cut in the flake chases and pinned only at the builder level; the 2026-09-29 review refused that cut, and the test is restored on this branch with the three non-Europe legs answering 404 from a live fixture.

## Evidence

- Branch head `f86a548e`; merge `26538f27`. The branch-head run has expired in the Actions API's retrievable window; no run ID exists to cite for it. The chain's final green state is 3d293ad7 → CI 36566765215 (success), and the 0.9.1-candidate main runs 36687060448 and 36693388098 are green with this code in them.
- Code review: ACCEPT 2026-09-29 by a fresh reviewer (dispatch 4304c685-8ea1-42b8-93ea-f7258988f683) after three REJECT passes whose findings were folded; the verdict history lives in the ticket file.
