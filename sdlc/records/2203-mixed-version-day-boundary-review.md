# Mixed-version cache review

Mixed v0.9.2/current use can leave the cache over the current size limit until maintenance. The PM accepted that compatibility boundary and retained the fast path without a per-operation scan. A finite follow-up completed two production daily-clean invocations with controlled due markers; both ended with exact saved totals and content below the limit. Ordinary eviction had already restored accounting before those two cleans, so this pass does not establish that the daily clean itself repairs a stale, over-budget cache.

## Inputs and execution

- Current source: `87d53c6782aaf07dcba4269f7dbd573a5ef00b58`; reused qualified executable `biomcp 0.9.4-dev+g87d53c67`, SHA-256 `31df21871fbf9ca95580f2ac15a05d3bb8f53079672db177e5363b6a9d611a8f`.
- Old source: tag `v0.9.2`, commit `4049666f5cce2794f16cdbdf0ded49878e5bb147`; public `biomcp-darwin-arm64.tar.gz` archive checked against its published SHA-256 sidecar. Extracted executable reports `biomcp 0.9.2`, SHA-256 `521c7ffebbb5a4749975966b04e76eb3ca0f12b6ebe8d38ac558ec42456acc0e`.
- Platform: macOS arm64. Build job `2203/1792355882950-20969` exited 1 after 6.597 seconds at the experiment's size assertion. No product rebuild ran.
- Isolation: one empty owned cache, isolated home/configuration/credentials through existing `retained_cache_env`, and the existing synthetic loopback gene provider from `tests/test_cache_persistent_size.py`. Each response includes 16,000 bytes of padding. `BIOMCP_CACHE_MAX_SIZE=50000`, `BIOMCP_CACHE_MAX_AGE=86400`, `BIOMCP_CACHE_MIN_DISK_FREE=1B`, cache mode `default`. No real provider or production data was used.

## Initial result and accepted boundary

Each command was a separate process: the selected executable followed by `--json search gene QUERY --limit 1`. All three returned exit 0, empty stderr, and a JSON result naming BRAF, `cached BRAF`, Entrez identifier `673`.

| Executable | Query | Physical content bytes after exit | Saved total after exit |
| --- | --- | ---: | --- |
| Current | `BRAFseed` | 17,440 | `v2 17440` |
| v0.9.2 | `BRAF1old1` | 34,882 | `v2 17440` |
| Current | `BRAF1current1` | 52,332 | `v2 34890` |

The old write left the current total stale. The following current write completed while actual content exceeded the 50,000-byte budget by 2,332 bytes. Its total omitted exactly the old write's 17,442 bytes. The startup-clean marker remained fresh and unchanged at `1792355890304` throughout; the script had not aged it.

The mechanism matches the source: v0.9.2 computes a process-local estimate and never updates the newer saved-total record. Current `src/cache/manager/totals.rs::add_content_bytes` trusts a valid existing record when adding new bytes. The experiment demonstrates undercount and an over-budget mixed-version window; it does not demonstrate a failed daily reconciliation, lost provider answer, or corrupted cache entry. The PM subsequently permitted this legacy-write window until daily maintenance; 2223 documents that boundary without a product-code repair.

## Controlled daily-clean follow-up

Job `2203/1792356717813-28154` passed in 1.008 seconds. It reused both verified executables without downloads or rebuilding. The original loopback server had stopped and its ephemeral port was embedded in cache keys, so the authorized follow-up used one fresh isolated cache and provider, preserving the original failing cache and results unchanged. A preceding setup-only job, `2203/1792356699681-26929`, exited 2 in 0.141 seconds because shell quoting had prevented creation of its script; it ran no product command.

The following thirteen separate CLI processes all returned exit 0 and the BRAF result. Only the mixed legacy window permits intermediate size overages; both controlled clean outcomes retain strict size and exact-total assertions.

| Step | Executable and query | Physical bytes | Saved total |
| --- | --- | ---: | ---: |
| Initial | Current `BRAFseed` | 17,440 | 17,440 |
| Cycle 1 old write | v0.9.2 `BRAF1old1` | 34,882 | 17,440 |
| Cycle 1 current write | Current `BRAF1current1` | 52,332 | 34,890 |
| Cycle 1 old write | v0.9.2 `BRAF1old2` | 52,334 | 34,890 |
| Cycle 1 current write | Current `BRAF1current2` | 34,892 | 34,892 |
| Cycle 1 warm control | Current `BRAF1current2` | 34,892 | 34,892 |
| Cycle 1 controlled daily clean | Current `BRAF1current2` | 34,892 | 34,892 |
| Cycle 2 old write | v0.9.2 `BRAF2old1` | 52,334 | 34,892 |
| Cycle 2 current write | Current `BRAF2current1` | 34,892 | 34,892 |
| Cycle 2 old write | v0.9.2 `BRAF2old2` | 52,334 | 34,892 |
| Cycle 2 current write | Current `BRAF2current2` | 34,892 | 34,892 |
| Cycle 2 warm control | Current `BRAF2current2` | 34,892 | 34,892 |
| Cycle 2 controlled daily clean | Current `BRAF2current2` | 34,892 | 34,892 |

Before the first controlled clean, the script changed only the startup marker to `1792266718827`; the production command rewrote it to `1792356718834`. Before the second, it changed the marker to `1792266718944`; the command rewrote it to `1792356718951`. Each controlled age was 25 hours. Both saved totals were exactly `v2 34892\n`, matching actual content and remaining below 50,000 bytes. Nine distinct queries with 16,000-byte padded replies caused real retention pressure and observed eviction.

## Bound and limits

The first attempt stopped after three commands under its original strict size criterion. The PM then accepted intermediate mixed-version overages and authorized one finite thirteen-command pass. Both planned production daily cleans ran successfully under controlled due-marker state. No actual 48-hour passage, entry-expiry clock behavior, long-lived process, concurrent versions, or forced termination was tested. Because each clean began with an already exact, in-budget cache, its ability to restore those properties from a stale, over-budget state remains unproved here.

Existing current-only short-process accounting and bounded-eviction evidence from 2195 and 2206 remains applicable to those scenarios; this mixed-version result adds a different writer. This report changes no product behavior and introduces no per-operation scan.

Owned experiment `2077` retains the original plan, script, verified executables and archive sidecar, artifact identities, isolated failing cache and raw results unchanged. Its `followup` folder holds the authorized later plan, script, fresh cache and every per-command stdout/stderr and byte/marker snapshot. Ownership remains `biomcp/2203` through review. Both loopback servers stopped when their experiments exited; delete disposable copies once the accepted finding is recorded and no open ticket consumes them.
