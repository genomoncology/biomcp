# 2091 — Cache GWAS and openFDA answers for a day

Status: OPEN.

Milestone: 0.9.3

Reviews: revision 68b2d71, accept

## Outcome

A repeat of the same GWAS Catalog query, openFDA label search or openFDA adverse-event count within 24 hours is answered from BioMCP's cache without a network request, and one after 24 hours goes to the provider again; `--no-cache` and `BIOMCP_CACHE_MODE=off` still bypass the cache.

## Evidence

- The run-time investigation on the build host (2026-10-14) found these three request kinds are the only slow calls left in warm runs of `search all --gene BRAF --disease melanoma` and `get drug imatinib`, and none is stored: the GWAS Catalog and openFDA both answer `cache-control: no-cache, no-store`, and BioMCP marks every GWAS request no-store besides. With them cached, a warm `search all` would fall from about 10 s to about 2 s and a warm `get drug` to well under a second.
- openFDA requests are already GET (`get_json_optional` in `src/sources/openfda.rs`), so no method change is needed. CIViC and Open Targets are GraphQL POSTs, which the cache library never stores; they stay uncached and out of scope.
- GWAS was made no-store in March 2026 (commit d616a957d) because "GWAS responses occasionally produce cache decode failures when a stale body entry is reused"; no reproduction was recorded. Ticket 1268 then pinned the rule against `BIOMCP_CACHE_MODE=infinite` with `src/sources/gwas/tests/no_store.rs` and `tests/test_gwas_no_store_contract.py`.
- The library's `CacheMode::IgnoreRules` stores a 200 regardless of headers but serves it with no expiry (http-cache 0.20.1), so BioMCP must enforce the 24 hours itself.

- PM decision 2026-10-14: GWAS Catalog association searches, openFDA label searches and openFDA adverse-event counts are held for 24 hours despite their no-store headers. Both sources change slowly, and a day-old answer is acceptable for a clinician's lookup. Requests sent with an openFDA API key stay no-store under the existing authenticated-request rule (`apply_cache_mode_with_auth`); this ticket does not change that rule.
- PM decision 2026-10-14: because the GWAS decode failure was never reproduced, a stored GWAS body is served only if it decodes; a stored body that fails to decode is discarded and the request goes to the network, and the fresh answer replaces it. That removes the failure class instead of relying on no-store.

- Starts from: `GwasClient::request_no_store` and `get_json_optional` in `src/sources/gwas.rs`; `OpenFdaClient::get_json_optional`, `label_search` and `faers_count` in `src/sources/openfda.rs`; `apply_cache_mode_with_auth`, `current_cache_mode` and the `HttpCacheOptions` built in `build_http_client_with_config_and_manager` in `src/sources/mod.rs`; the cache-behaviour section of `docs/reference/configuration.md`; `src/sources/gwas/tests/no_store.rs`; `tests/test_gwas_no_store_contract.py`.
- Keeps: every other source's cache behaviour; `--no-cache`, `BIOMCP_CACHE_MODE=off` and authenticated requests bypassing the cache; the response bodies BioMCP prints.
- Changes: claims `src/sources/gwas.rs` `src/sources/gwas/tests/no_store.rs` `src/sources/gwas/tests/mod.rs` `src/sources/openfda.rs` `src/sources/mod.rs` `tests/test_gwas_no_store_contract.py` `docs/reference/configuration.md` `CHANGELOG.md`.
  - A day-cache request kind for these three request plans: stored on a 200 whatever the headers say, served while the stored entry is under 24 hours old, refetched after. The developer picks the mechanism (an `IgnoreRules` request mode with an age check on read, or the library's per-request mode hook) and says which in the review.
  - Replace the GWAS no-store pin and its text contract with tests of the new rule; delete what no longer applies.
  - One sentence in `docs/reference/configuration.md` naming the three request kinds held for a day.
  - One New features changelog bullet.
- Proof: routine suite. Tests against a local server that counts requests: a second identical GWAS search, openFDA label search and openFDA adverse-event count within 24 hours make no request; with the stored entry aged past 24 hours (by an injected clock or by rewriting its stored time) they make one; under `--no-cache`, `BIOMCP_CACHE_MODE=off` and with an openFDA key they always make one; a stored GWAS body that fails to decode is replaced by a fresh fetch and the command succeeds. The first two fail on current main. On the build host the reviewer records warm `search all --gene BRAF --disease melanoma --counts-only` and `get drug imatinib` medians before and after.
- Defers: caching CIViC and Open Targets POSTs; per-source freshness settings for users.
- Order: after 2089 and 2090, which change the same call paths for concurrency. It edits `src/sources/mod.rs`, which no other open 0.9.3 ticket claims.
