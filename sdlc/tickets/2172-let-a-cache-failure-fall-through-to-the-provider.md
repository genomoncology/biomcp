---
priority: 2
---

# 2172 — Let a cache failure fall through to the provider

Status: OPEN.

Milestone: 0.9.3

Depends on: 2170

Reviews: revision a3b894d, accept

## Outcome

A failure in the HTTP cache never makes a provider look unavailable: a cache setup, read or write failure logs a local cache warning and the request goes to the provider uncached, or keeps the provider's answer it already has.

- Evidence: found by the sweep after GitHub issue 292, where every run succeeded with `--no-cache`. Client construction propagates every cache step with `?` (`src/sources/mod.rs:1374-1394` and `1463-1500`: epoch lock, shared operation lock, securing the tree, start-up clean through `src/cache/manager.rs:138` and `524-542`), so any local cache problem aborts the client and the source reports unavailable. A cache read that fails to secure the tree is an error (`src/cache/manager.rs:263-290`), and an entry that fails bincode decoding propagates through http-cache. The cache write (`src/cache/manager.rs:308-331`) propagates key-lock, write and commit errors after the provider has answered, so a full disk or a lock timeout throws away a good response.
- Starts from: `src/cache/manager.rs`, the cached client construction in `src/sources/mod.rs`, and the cache deadline tests.
- Keeps: deadlines, the cache's security checks on its own tree (a tree that fails them is bypassed, never trusted), cancellation safety around publication, `--no-cache`, and every existing cache hit.
- Changes: claims `src/sources/tests/provider_network.rs` `src/entities/article/search/tests/deadline.rs` `tests/test_http_cache_fail_soft.py` `src/cache/manager.rs` `src/sources/mod.rs` `src/cache/` `CHANGELOG.md`. When cached construction fails, build the uncached client and warn once with the local cause. Treat a read failure or an undecodable entry as a miss. Return the provider's answer when the write fails, and warn.
- Proof: outside-in, through `search article` and one other entity command against the recorded-reply server. On current main, a held epoch lock past the deadline, an undecodable entry and a failed write each make the source fail or report unavailable. After the change, each returns the provider's answer and logs one local cache warning naming the cause.
- Defers: retrying Windows sharing violations on rename and remove.

## Implementation brief

Restated outcome at 17f4db007: an optional HTTP cache failure must not hide a working provider or discard its successful answer. Reproduce through real article and gene searches using synthetic recorded replies, covering insecure/invalid cache admission, corrupt payload and write obstruction, plus retained cache hits and no-cache behavior. Add tests/test_http_cache_fail_soft.py; product changes remain the cache manager and cached-client construction. The brief's observed propagation sites supply the initial inspection evidence; record executed red commands before implementation.

Coordinator decision: cache request paths try to admit optional cache work without waiting away the provider deadline. On contention, bypass the cache and warn locally; on cache write contention/failure, keep the obtained provider response. Keep explicit cache maintenance locking and essential local-dataset locking unchanged. Preserve tree security checks, direct lock-contention mapping, and the end-to-end provider deadline. Update construction-contention proof to require prompt uncached provider fallback within that deadline, rather than a cache-construction timeout. This introduces no new cache wait duration or configuration setting.

## Reproduction on current main

On 17f4db007, corrected red build job 1792257293028-97204 used the normally built routine binary after applying only the test patch. The inner command was:

```sh
mkdir -p "$PWD/.cache/routine-test-tmp"
export TMPDIR="$PWD/.cache/routine-test-tmp"
BIOMCP_BIN="$PWD/target/debug/biomcp" uv run --no-sync pytest tests/test_http_cache_fail_soft.py -q
```

Result: six failed, one passed in 10.22 seconds. Both invalid-cache-root cases (article and gene) fail with initialization errors. The valid-integrity invalid-bincode entry and post-provider write-failure cases fail with http_middleware. Held body-limit and operation locks consume the three-second invocation deadline and return source_unavailable. The no-cache case passes without modifying the rejected cache state. An earlier preparation run's fixture mismatch was corrected before this reproduction; it is not part of the red evidence.

Deadline interpretation: explicit invocation expiration or cancellation remains final; never start provider work after its deadline. Ordinary local cache errors, including local timeouts while invocation budget remains, fail soft. Keep publication cancellation safety, and report any case where an outer deadline can still discard an already-obtained answer while cache work settles.

## Affected checks

Developer source db84f476fbf81f1bf54affb6172791fde9146e53 is ready for fresh review. Job 1792257536468-48301 passed 96 affected Rust tests, including cache security and maintenance behavior, HTTP write/finalization response retention, invocation expiry, and article fallback with a held epoch lock. Job 1792257759015-16175 confirmed the unreadable configuration case fails against the old main binary, then passed ten unaffected Python cases against the new binary. Job 1792257917040-39513 passed all three article cases after correcting only the recorded fixture pagination, and passed the quality ratchet. Final formatting and all-target routine Clippy with warnings denied passed in 1792257930740-41306. Reuse these focused results; the reviewed merge candidate still requires its landing gate.

## Fresh review

The fresh reviewer accepts source 15f83e28e2b61c5f7576af942d72b3f7a15bdb3b with two fixes. Missing indexed content and integrity failures now reach the optional adapter for a warning, while absent entries remain normal misses; red job 1792258114232-65093 failed both warning cases and green 1792258139951-67305 passed ten Python cases, four relevant Rust cases, formatting and Clippy. A retained response after successful 304 revalidation now drops the stale marker before cloning: red 1792258467441-19268 failed its new assertion and green 1792258538494-23343 passed the revalidation and publication-deadline cases, formatting and Clippy.

Preserved limit: invocation expiry before the publication marker arms during writer creation or write_all remains final, even if the provider body already arrived. Slow filesystem calls remain outside this fix. The request-path lock bypass and local I/O fallback do not remove the end-to-end invocation deadline.

## Integration repair brief

Shared gate 1792258704769-36960 passed lint and 1249 Python cases, then failed a_stale_search_all_json_states_the_cache_age_in_meta_notes and a_stale_article_search_json_states_the_cache_age_in_meta_notes. Both report remote request errors instead of the existing stale fallback result. The preceding four-ticket candidate without 2172 passed its Rust suite in gate 1792258265870-81665, so treat this as a cache integration regression until evidence says otherwise. Reproduce both on this published branch, preserve stale fallback and its truthful age notes, and fix the adapter/read path rather than weakening assertions.

The coordinator separated 2172 with integration revert 1c3a0b072 on the provider batch. That revert will be in main. Before re-landing this ticket, integrate that main and reverse the integration revert to restore the full reviewed cache implementation together with this repair; merging the old branch alone will not restore reverted code. Keep the ticket open until the restored candidate is reviewed and gated.
