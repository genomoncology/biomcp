# Mixed-version cache review

The bounded review found a size-limit breach after alternating the current CLI with v0.9.2. It stopped at that first product discrepancy, before either planned controlled daily clean. Daily-clean reconciliation remains untested by this experiment.

## Inputs and execution

- Current source: `87d53c6782aaf07dcba4269f7dbd573a5ef00b58`; reused qualified executable `biomcp 0.9.4-dev+g87d53c67`, SHA-256 `31df21871fbf9ca95580f2ac15a05d3bb8f53079672db177e5363b6a9d611a8f`.
- Old source: tag `v0.9.2`, commit `4049666f5cce2794f16cdbdf0ded49878e5bb147`; public `biomcp-darwin-arm64.tar.gz` archive checked against its published SHA-256 sidecar. Extracted executable reports `biomcp 0.9.2`, SHA-256 `521c7ffebbb5a4749975966b04e76eb3ca0f12b6ebe8d38ac558ec42456acc0e`.
- Platform: macOS arm64. Build job `2203/1792355882950-20969` exited 1 after 6.597 seconds at the experiment's size assertion. No product rebuild ran.
- Isolation: one empty owned cache, isolated home/configuration/credentials through existing `retained_cache_env`, and the existing synthetic loopback gene provider from `tests/test_cache_persistent_size.py`. Each response includes 16,000 bytes of padding. `BIOMCP_CACHE_MAX_SIZE=50000`, `BIOMCP_CACHE_MAX_AGE=86400`, `BIOMCP_CACHE_MIN_DISK_FREE=1B`, cache mode `default`. No real provider or production data was used.

## Observed failure

Each command was a separate process: the selected executable followed by `--json search gene QUERY --limit 1`. All three returned exit 0, empty stderr, and a JSON result naming BRAF, `cached BRAF`, Entrez identifier `673`.

| Executable | Query | Physical content bytes after exit | Saved total after exit |
| --- | --- | ---: | --- |
| Current | `BRAFseed` | 17,440 | `v2 17440` |
| v0.9.2 | `BRAF1old1` | 34,882 | `v2 17440` |
| Current | `BRAF1current1` | 52,332 | `v2 34890` |

The old write left the current total stale. The following current write completed while actual content exceeded the 50,000-byte budget by 2,332 bytes. Its total omitted exactly the old write's 17,442 bytes. The startup-clean marker remained fresh and unchanged at `1792355890304` throughout; the script had not aged it.

The mechanism matches the source: v0.9.2 computes a process-local estimate and never updates the newer saved-total record. Current `src/cache/manager/totals.rs::add_content_bytes` trusts a valid existing record when adding new bytes. The experiment demonstrates undercount and missed size enforcement after an old write; it does not demonstrate a failed daily reconciliation, lost provider answer, or corrupted cache entry.

## Bound and remaining proof

The approved plan allowed thirteen commands with nine distinct queries and two real current daily-clean invocations, each made due by aging only the owned startup marker by 25 hours. The coordinator required stopping at the first product failure. Three commands ran and no marker was aged, so neither planned daily clean ran. No actual 48-hour passage, entry-expiry clock behavior, long-lived process, concurrent versions, or forced termination was tested.

Existing current-only short-process accounting and bounded-eviction evidence from 2195 and 2206 remains applicable to those scenarios; this mixed-version result adds a different writer. A separate repair must address this failure before the remaining controlled daily-clean proof can be completed. This report changes no product behavior.

Owned experiment `2077` retains the plan, one-shot script, verified executables and archive sidecar, artifact identities, isolated failing cache, per-command stdout/stderr and byte/marker snapshots. Keep it while the repair consumes this evidence, then remove its disposable copies. The loopback server stopped when the experiment exited.
