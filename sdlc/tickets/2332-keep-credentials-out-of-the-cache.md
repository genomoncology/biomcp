# 2332 — Keep credentials out of the cache and logs

Status: COMPLETE.

Milestone: 0.9.4

Reviews: revision 55b3377f0d1484b5dbd2693bbcd667f1b9cddaa6, reject

Reviews: revision 66730a44f2277159303a076a57a8aeea4cbe66f6, accept

Reviews: revision 629ec354b4c966a3cc710a4b40ca526f12154d85, accept

Reviews: revision d51d921e75aca4ca0738f74e857b2889075be890, accept

## Outcome

No credential BioMCP sends reaches disk or logs. That covers the NCBI api_key query parameter, other key or token query parameters, and authorization or x-api-key headers.
- The SQLite cache stores no credential in any column, including the serialized cache-policy metadata, which may carry the original request URL and headers. The blobs directory stores none either.
- Debug output, error messages and logs show credentials redacted.
- On first cache open, 0.9.4 deletes the legacy 0.9.3 stores (http, http-cacache and the other LEGACY_STORES) under the cache root. Their cacache index used full request URLs, including api_key, as keys. 0.9.4 never reads them.
- Any credential already in a 0.9.4 database is dropped or rewritten on first cache open.
- PM ruling m3046: preserve ordinary `--no-cache` and cache-mode-off behavior. Both `cache clean` and confirmed `cache clear` remove the legacy stores.

- Proof: an outside-in test runs keyed NCBI, Semantic Scholar and header-key requests against the replay server with fake canary keys. A byte search of the whole cache root, stdout and stderr, with debug on, then finds no canary. A test seeds a legacy http-cacache store holding a canary URL and shows it gone after the first command. The sweep is idempotent and leaves the SQLite cache intact.

Changes: `CHANGELOG.md`, `docs/policies.md`, `docs/reference/configuration.md`, `docs/reference/data-sources.md`, `docs/troubleshooting.md`, `docs/user-guide/cli-reference.md`, `src/cache/http.rs`, `src/cache/manager.rs`, `src/cache/mod.rs`, `src/cache/sqlite.rs`, `src/cache/sqlite/request_key.rs`, `src/cache/sqlite/schema.rs`, `src/cache/sqlite/upgrade.rs`, `src/cli/cache.rs`, `src/cli/cache/tests.rs`, `src/sources/mod.rs`, `tests/cache_clear.rs`, `tests/feature-links.json`, `tests/managed_state_permissions.rs`, `tests/test_cache_credentials.py`, `tests/test_cache_sqlite.py`.

## Evidence

- Current successful keyed PubMed requests already bypassed persistence; their baseline control passed. Reproduced credential-persistence failures concern retained legacy stores and pre-fix SQLite rows/blobs. All new probes use fake canaries, not actual credentials.
- Fresh review rejected source `55b3377f0d1484b5dbd2693bbcd667f1b9cddaa6`: actual-entry two-CLI proof j1975 showed a delayed initializer deleting two rows published after another initializer completed. The private upgrade lease now owns cleanup through schema marking and rechecks the schema under that lease. Published-source j2062 asserts `aa735c135f606f338d0590b095376422d937b383`, preserves both rows and proves subsequent reuse. Direct full routine j2030 passed on that source; fresh review accepted it. Native Windows Python coverage was not claimed.
- Authorized default-store cleanup on the three build hosts was completed without reading cache contents; saved downloads, sessions, active job/evidence roots and Research copies were preserved. Ordinary no-cache behavior and confirmed cache clean/clear follow PM m3046.

- On 2026-10-21, research found the NCBI key in 1,823 files on Blue and 1,244 in Yellow's backup, every one as api_key= in an NCBI or PMC request URL used as an index key by BioMCP's HTTP cache (http/index-v5, the 0.9.3 cacache store). Semantic Scholar's key appears nowhere.
- 0.9.4's request key strips api_key, apikey, api-key, key and access_token before hashing (src/cache/sqlite/request_key.rs:24-31). RequestContext still keeps original_url and the headers, and the entries table stores a metadata BLOB (src/cache/sqlite/schema.rs). Confirm whether that metadata holds the original URL or headers.
- src/cli/cache.rs lists LEGACY_STORES and offers a legacy clear, but upgrades do not delete those stores automatically.
- Ian wants every fix in 0.9.4. BioMCP is public, so this reaches every user who sets NCBI_API_KEY.
