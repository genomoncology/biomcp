# Search-all GWAS latency and completed-response reuse

The observed timeout is an upstream delay. Preserve the approved two-query, bounded rsID intersection and both 50-row requests. The existing code reports a real provider timeout truthfully; the captured pages produce a successful empty intersection when served promptly. The PM approved closing this investigation without a product change after the completed-response day-cache check passed.

## Source and command

Product source: `096c11fc416ddc18ee3b05814a6d0b9fede63648`, with qualified CLI SHA256 `e47d903c9a83a1c8d18d9cd208e20ee4a27a03caa1f0e1a72b075a42542d949d`. Study branch `7a13967086d8ca1506a1f28ca92cd35661fc6b65` differs only in ticket records. The cache check uses ruling revision `a800ca972d835249d73f7760f0e781821d8fec25`, with the same qualified product binary. These are observations of that snapshot, not final release measurements.

```bash
biomcp --json search all --gene BRAF --disease melanoma --counts-only
```

## Live observation

Job `2232/1792416166291-16096` ran this command once with other providers live and a forwarding GWAS recorder. The CLI exited 0 in 12.02754 seconds and reported `gwas search timed out after 12s`. The two requests began within 2.4 milliseconds.

| Request | Headers received | Complete body | HTTP status |
| --- | ---: | ---: | ---: |
| `v2/associations?page=0&size=50&sort=p_value&direction=asc&mapped_gene=BRAF` | 17.03280 s | 17.22159 s | 200 |
| `v2/associations?page=0&size=50&sort=p_value&direction=asc&efo_trait=melanoma` | 10.51024 s | 10.64974 s | 200 |

This establishes a genuinely slow BRAF request. It does not establish that page size caused the latency or that a smaller page would finish sooner.

The full live bodies match the existing frozen recordings byte for byte:

| Recording | SHA256 |
| --- | --- |
| `testdata/replay/responses/gwas/gwas-v2-associations-863182df44d8.bin` | `2e979e4667602c820bc69ae2c9be92719ba8ce89878ee4262d3ed469bc98557c` |
| `testdata/replay/responses/gwas/gwas-v2-associations-01e578363684.bin` | `412bd6fe2c3b012d691ec028364ca45fbe7e8969cc0a1a7e0beb5231ab559aa5` |

## Replay and query meaning

Prompt recorded responses returned GWAS `returned: 0`, `total: 0`, without error, in 3.83029 seconds. These two bounded pages share no rsIDs; the existing counts-only golden expects that empty result. Standalone default `search gwas -g BRAF` requests only the gene leg with size 11, so it uses another cache key. There is no exact size-11 recording, and its replay miss supplies no latency comparison. Standalone `--limit 49` uses the recorded size-50 URL and returns 49 rows.

Job `2232/1792416249863-28418` served the original recorded bodies while delaying the BRAF leg by the observed 17.22159 seconds. The CLI reproduced the 12-second timeout (`wall_time_ms: 12004`); the delayed recorded reply ultimately completed. This is an explicit model of the observed latency, not evidence of historical server timing beyond the live collection.

The clinical review confirmed the existing bounded-intersection approvals in [record 0928](0928-make-gwas-filter-semantics-truthful.md) and [ticket 2089](../tickets/2089-run-search-alls-two-gwas-queries-at-the-same-time.md). Reducing a page or removing a query changes that candidate set or question. An empty intersection of captured pages does not establish absence of an association in the complete Catalog: the [Catalog API](https://www.ebi.ac.uk/gwas/docs/programmatic-access/rest-api/) is paginated. No retrieval-semantic change was authorized. A diagnostic that forced a nonempty result supplied no valid product regression and is excluded from the evidence of a defect.

## Completed-response day-cache reuse

Job `2232/1792416572833-25655` executed two identical search-all commands in an isolated ordinary cache. The first completed both recorded GWAS responses. The GWAS fixture then switched to HTTP 503 for any later request.

| Call | New GWAS requests | GWAS result | Command elapsed |
| --- | ---: | --- | ---: |
| First | 2 | Successful empty intersection | 4.520 s |
| Second | 0 | Same successful empty intersection | 3.050 s |

The second call reused both completed bodies without another provider wait. This makes no promise that a cold request canceled before its body arrives creates a cache entry.

## Retained evidence

Thirty raw request, header, full-body, CLI-output and completed job-record files are retained in `job-runs/biomcp/2232/1792416166291-16096/2232-study-evidence.tar.gz`, SHA256 `050c4cd67cca54d2e4f3b041a22ebbaa2cc795cacd641f1e8e0f35e4803323a1`. Each archived member was compared byte for byte with its original. Job logs remain. Owned copied binaries, scripts and servers were removed; unique raw copies remain until the record lands and cleanup verifies the archive.
