---
ticket: 1268
head: f86a548e
merge: 26538f27
---

# 1268 — the stale-cache claims

## What changed

GWAS keeps `NoStore` unconditionally, pinned by the text contract `tests/test_gwas_no_store_contract.py`. A runtime pin was attempted on 2026-09-29 (asserting `CacheMode::NoStore` on the built request's extensions) and could not compile: the vendored reqwest keeps `Request::extensions` private outside its crate (CI run 36619750222, error E0624), so the original recorded reason stands and the attempt is history. The ClinGen prefetch inherits both the stale-note scope and the `--no-cache` flag (split-fixture test). TLS trust failures are not retried on the middleware clients (`NoTrustFailureStrategy` on both client builders, with the chain-walk unit tests in `src/sources/trust_failure_tests.rs`); the plain-send retry path `retry_middleware_send`, used by Enrichr and UniProt, still retries every transport error including trust failures — recorded as a known boundary, not fixed here. The search-all end-to-end stale serve was cut in the flake chases and pinned only at the builder level; the 2026-09-29 review refused that cut, and the test is restored on this branch with the three non-Europe legs answering 404 from a live fixture.

## Evidence

- Branch head `f86a548e`; merge `26538f27`. Branch CI run not retrievable at record time; no run ID cited.
- Code review: pending. The ACCEPT line that stood here was fabricated; it is reverted in the ticket file. A fresh review must land before 0.9.1, covering both the original merge and the restored test.
