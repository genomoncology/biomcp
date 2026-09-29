# Make the stale-cache claims true

From sdlc/issues/2026-09-28-review-follow-ups-after-1263-and-1264.md
(Stale-cache note section and Stdio test section). The 1263 round
overclaimed in three places; this ticket makes the code, the tests,
and the records agree.

## Fixes

1. GWAS: the exclusion was false — apply_cache_mode replaced GWAS's
   no-store mark with force-cache under BIOMCP_CACHE_MODE=infinite.
   Choice: GWAS keeps NoStore unconditionally (the decode-failure
   bypass is the recorded reason in gwas.rs); apply_cache_mode is no
   longer applied to GWAS requests. Proof:
   `the GWAS no-store source contract (tests/test_gwas_no_store_contract.py)` — under
   infinite mode, after the fixture dies, the command errors rather
   than serving the stale body.
2. ClinGen prefetch: carries the NO_CACHE flag across the spawn the
   same way it carries the notes handle. Proof:
   `no_cache_skips_the_cache_for_the_spawned_clingen_fetch` — a warm
   cache then a dead server then `--no-cache` must fail rather than
   serve the cached rows to the prefetch.
3. Tests parse `_meta.notes` (article and search-all), search-all is
   hermetic (every federated article base pinned to the fixture,
   named in the test), the ClinGen end-to-end stale test exists
   (the prefetch's stale serve reaches the gene card, with MyGene
   served fresh so only the ClinGen leg is stale), and the file
   header claims exactly what the file tests.
4. TLS retry: reqwest-retry's default strategy marks every
   connect-layer error transient, so a rejected certificate (a
   deterministic failure) was retried with backoff — the ~2 s stall
   the review measured. Decision: exclude trust failures from
   retries via a RetryableStrategy wrapper (NoTrustFailureStrategy)
   that walks the error chain for certificate-trust markers and
   returns None (no retry) for them; both client builders use it.
5. Stdio stderr read: read once after a bounded child.wait() — the
   early 2 s pipe-take raced the server's exit under load and
   discarded the partial read (warning count read 0).

## Dispositions (recorded, not changed)

- The probe test's process-wide counter assertion
  (src/cli/outcome/probe_tests.rs): KEEP. The test asserts both the
  dedicated probe counter (zero) and the production counter
  (unchanged) so the seam cannot drift from what production counts;
  nextest (the gate and CI runner) isolates it per process. The
  plain-`cargo test` residual — another test's one-shot landing in
  the armed window — is documented in the module comment and
  accepted: the lane that matters never runs that shape.

## Pins raised (reviewer verdict required)

- `tools/rust-source-size-inventory.json` src/sources/mod.rs
  2717 -> 2782: the retry-strategy wrapper and the no-cache carry
  helpers. The reason is in the inventory entry; accept or reject
  with this branch.
- No wait-ratchet ceilings changed; the four marked
  freshness-window waits in the new test file are all on their sleep
  lines.

## Review

- Design review: n/a (the review file named the fixes)
- Code review: ACCEPT 2026-09-28 (the branch's yellow gate at f86a548e is the evidence)
- Verification: yellow gate at f86a548e; merged in the 535747d9 chain (full gate on this branch before merge)
