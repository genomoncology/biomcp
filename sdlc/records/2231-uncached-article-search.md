# Uncached article-search recovery cost

Retain the approved visible-row metadata recovery. The uncached timing gap reproduces, and the timing breakdown attributes the added work to Europe PMC recovery after PubTator metadata resolution fails. The older release stopped at the warning. Reverting that recovery would remove behavior approved in ticket 2088. This outcome records the cause and why it stays; it proposes no product change.

## Comparable inputs

Old source: v0.9.2, commit `4049666f5cce2794f16cdbdf0ded49878e5bb147`. Current source: `fb513f102f807e2599168b9d07b2491684d9c59d`, whose product files match main `096c11fc416ddc18ee3b05814a6d0b9fede63648`; only ticket records differ. Both builds used `--release --locked --no-default-features` and the same host compiler settings. The archive retains exact source and binary identities, build commands and logs.

The existing measurement helper's run_once function is byte-identical to the original frozen helper at 631f7904. The study ran five alternating old/current pairs of the original uncached article CLI, followed by five alternating pairs with full JSON diagnostics. Each used `BIOMCP_CACHE_MODE=off` and `search article -g BRAF --limit 10`; the latter added global `--json` and article `--full`. No inherited NCBI key or compiler override names were present.

The completed collection is `2231/1792416407642-68922`. All twenty processes exited successfully without timeout. The host was shared, with load ranging from 8.60 to 10.30; providers remained live. These are paired observations of this snapshot, not quiet-host or final-release measurements. An earlier prerequisite attempt stopped before timings because the old identity wrapper selected a Python without tomllib; using the canonical prepared Python completed the old build without rebuilding the completed current artifact.

## Measurements

| Observation | v0.9.2 median | Current median | Median paired increase |
| --- | ---: | ---: | ---: |
| Original uncached CLI | 3.44449 s | 4.98355 s | 1.79848 s |
| Full JSON diagnostics | 3.08357 s | 4.59392 s | 1.51267 s |

The current gap is larger than the historical roughly 0.45-second observation. Live-provider and shared-host conditions differ; the study does not claim a fixed universal overhead.

| Diagnostic stage | v0.9.2 median | Current median |
| --- | ---: | ---: |
| Metadata fallback | 719 ms | 2247 ms |
| PubMed search | 1478 ms | 1482 ms |
| PubTator search | 950 ms | 878 ms |
| Semantic Scholar enrichment | 648 ms | 619 ms |

The five paired added metadata costs were 1217, 1723, 1542, 1041 and 1353 milliseconds. Across each diagnostic pair, the search, enrichment and metadata stage differences account for the wall-time difference within two milliseconds. Search-provider variation and Semantic Scholar enrichment do not explain the added cost. No keyed NCBI pacing configuration was inherited, so the keyed pacing change is not the explanation for these samples.

All five diagnostic pairs retained ten rows, the same PMID order, the same 60-second deadline and two truthful metadata-fallback warnings.

## Cause and retained behavior

Source commit `31d388af5d7c78730cc5319c90f9d963eff1547c`, for ticket 2088, changes `src/entities/article/enrichment.rs`: after failed PubTator visible-row metadata resolution, current code calls Europe PMC search_by_pmid. The older release only warned. Each diagnostic sample had two failed PubTator metadata lookups; current therefore made two additional recovery attempts before retaining the warnings when recovery also failed.

This is the approved recovery behavior, rather than a slower initial search or a changed result-order policy. Keep the recovery source, deadline, ordering and honest error reporting. No fabricated failing product regression or source-file claim is warranted for this conditional record outcome. This study does not rule out a separately scoped behavior-preserving optimization; it establishes the cause of the measured difference and preserves the approved behavior.

## Retained evidence

The archive `job-runs/biomcp/2231/1792416407642-68922/measurement-evidence.tar.gz`, SHA256 `175f1cd85180eb2f8ed706f21e21392205846446cdb236ff347243cbde2a643f`, retains scripts, helper identity, exact source and binary identities, build logs, raw outputs, loads and paired breakdowns. Owned temporary old source/build files were removed after verification. Job logs remain. No product source was edited.
