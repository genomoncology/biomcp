# 2332 — Keep credentials out of the cache and logs

Status: OPEN.

Milestone: 0.9.4

## Outcome

No credential BioMCP sends reaches disk or logs. That covers the NCBI api_key query parameter, other key or token query parameters, and authorization or x-api-key headers.
- The SQLite cache stores no credential in any column, including the serialized cache-policy metadata, which may carry the original request URL and headers. The blobs directory stores none either.
- Debug output, error messages and logs show credentials redacted.
- On first use, 0.9.4 deletes the legacy 0.9.3 stores (http, http-cacache and the other LEGACY_STORES) under the cache root. Their cacache index used full request URLs, including api_key, as keys. 0.9.4 never reads them.
- Any credential already in a 0.9.4 database is dropped or rewritten on first use.

- Proof: an outside-in test runs keyed NCBI, Semantic Scholar and header-key requests against the replay server with fake canary keys. A byte search of the whole cache root, stdout and stderr, with debug on, then finds no canary. A test seeds a legacy http-cacache store holding a canary URL and shows it gone after the first command. The sweep is idempotent and leaves the SQLite cache intact.

## Evidence

- On 2026-10-21, research found the NCBI key in 1,823 files on Blue and 1,244 in Yellow's backup, every one as api_key= in an NCBI or PMC request URL used as an index key by BioMCP's HTTP cache (http/index-v5, the 0.9.3 cacache store). Semantic Scholar's key appears nowhere.
- 0.9.4's request key strips api_key, apikey, api-key, key and access_token before hashing (src/cache/sqlite/request_key.rs:24-31). RequestContext still keeps original_url and the headers, and the entries table stores a metadata BLOB (src/cache/sqlite/schema.rs). Confirm whether that metadata holds the original URL or headers.
- src/cli/cache.rs lists LEGACY_STORES and offers a legacy clear, but upgrades do not delete those stores automatically.
- Ian wants every fix in 0.9.4. BioMCP is public, so this reaches every user who sets NCBI_API_KEY.
