# 2234 — Compare the 0.9.4 runtime candidate with 0.9.3

## Decision and scope

The PM accepted the measured developer build costs for 0.9.4 and assigned follow-up 2284 to isolate and reduce test-profile compilation cost in 0.9.5. The PM also accepted two runtime costs and the small-cache storage cost as known trade-offs: roughly 5 ms for warm variant lookup, roughly 1 ms for cached GWAS search, and tens of kilobytes of fixed SQLite overhead per small cache. These observations are not established noise. Their timing cause remains unknown. The disposition came through `inbox/biomcp/2026-10-19-biomcp-dispose-final-runtime-timing-and-cache-costs-for-0-9-4.md`.

This experiment compares runtime source `51f5e03b218c87cedc82bd9e1a67552bd5ab9df8` (tree `90d9f94dfa5541a135bc728d88f849e595537d15`) with v0.9.3 `5641de7d7b79b53872f8599dbb4dba22e611dcaf` (tree `a00ac7632b785c692c33311387e717ef6aa70c6b`). The coordinator authorized reuse of this runtime source after test-only corrections 2283 and 2285. Qualification of the later exact release tip remains separate. This record does not authorize a tag.

## Retained evidence

Private build-host experiment `experiments/809-biomcp-2234-paired` and independent workspace experiment `experiments/491-biomcp-2234-raw` have OWNER `biomcp/2234` and ticket Keeps. Both retain `final-raw.tar.gz`: 1,484,806,694 bytes, SHA256 `f66134fc07d98ef7cc853e231a6ff826cf256763082fe5f55149b62b726a11fe`. Its manifest lists 930 files and 5,085,130,016 uncompressed file bytes. Independent verification checked the archive digest and every member size and SHA256. The archive includes all six arm snapshots, Cargo timing output, commands, safe environment/toolchain/profile/source metadata, failed attempts, all continuation scripts, job logs/state/results, release binaries, runtime stdout/stderr, source-disable proofs, cache bodies, disk inventories and stats. Regenerable source clones and Python environments remain separately retained on the build host. No unique raw data was discarded. The earlier historical 2214 raw archives were unavailable; this experiment did not recover them.

`commands.json` contains command arrays, exit codes, outer elapsed times, load and disk availability. Each `pairN-ARM-SECTION.log` contains the unchanged helper's canonical `RESULT` rows. `runtime-pairs.json` and each arm's `runtime/*/samples.json` preserve every runtime observation, command, order, watchdog result, output digest and load. Cache `*-disk.json` and `*-stats.json` retain individual files and logical payload statistics. `final-raw-manifest.json`, `final-raw-archive.json` and the independent `final-raw-verification.json` identify the retained bytes.

## Method

The existing `tools/measure-0-9-3-baseline` was reused unchanged, SHA256 `9c427657dcd0c79a1982aae3f692433a88a9b273f131a4ca62b86962aa4bc9c7`, following 2082 and 2214. Thin orchestration sequences its existing sections and adds the approved direct main-gate and cache-accounting commands. It does not replace the timing implementation. All builds, tests and samples ran through admitted M5 jobs. The same host, Rust/Cargo 1.95.0, Python 3.12.13, native Darwin linker, compiler cache and profiles served both arms. uv was 0.11.14. Exact toolchain/linker output is retained. Development settings use sccache, incremental=false, line-tables-only debug information and dependency debug=false. Release retains thin LTO, one codegen unit and stripping. No compiler/profile change hides a failure.

Build pair order was old/new, new/old, old/new. Each arm began with an independent empty target root; the configured compiler cache remained shared. Helper rows are cold development all-target build without default features and with Cargo timings; Rust-touch rebuild; documentation-touch rebuild; the existing gene edit-test group; prepared `make test`; prepared `make spec`; release build, binary size, target sizes and peak RSS. Preparation is recorded separately. The full main comparison is exactly `env -u CARGO_INCREMENTAL make main-gate`. Timed commands and source mutations remain in the retained helper and scripts.

Runtime uses the existing helper's `run_once`, exact workload and 180-second watchdog: five alternating pairs per row, ten for drug detail, with prefetch separate and independent per-arm warm/off/day roots. Each repeated workload alternates the first arm and executes warm then cache-off within that arm. There are 87 observations per arm: 80 measured and seven prefetch. Only `NCI_API_KEY` may inherit; no credential value was inspected, logged, hashed or retained. Other provider configuration, credentials and proxies were isolated. Authenticated NCI requests follow the existing uncached path; a warm search-all command can still contain live work.

Both runtime arms and every prefetch/cache workload use `BIOMCP_DISABLE_SOURCES=commercial-restricted,kegg,semantic-scholar`. Existing pure-local `--json health --api KEGG --api 'Semantic Scholar'` output proves both selected rows disabled in both binaries and lists the shared restricted source inventory as disabled. Candidate also lists its added ORCID restriction. No KEGG or Semantic Scholar provider call was authorized. Query arguments and pair order stayed unchanged; these source-filtered numbers have no asserted numerical continuity with historical unfiltered runs.

## Build and gate observations

Times are seconds. Samples within a cell follow build pairs 1, 2, 3. Change is the median of matched candidate/old ratios, not the ratio of marginal medians. The failed old test contributes no passing timing; that row has two complete pairs. Helper timing precision remains 0.1 second. Direct main-gate values below are rounded from retained outer elapsed values.

| Row | Old samples | Candidate samples | Complete pairs | Paired change |
| --- | --- | --- | ---: | ---: |
| Cold development build | 78.3 / 51.7 / 47.5 | 84.0 / 64.6 / 51.6 | 3 | +8.63% |
| Rust-touch rebuild | 54.7 / 39.9 / 38.8 | 58.5 / 55.5 / 43.1 | 3 | +11.08% |
| Documentation-touch rebuild | 0.3 / 0.3 / 0.3 | 0.3 / 0.3 / 0.6 | 3 | 0.00% |
| Gene edit-test group | 53.9 / 39.8 / 39.7 | 63.2 / 58.9 / 42.4 | 3 | +17.25% |
| Prepare-test context, outside test timing | 5.0 / 4.0 / 3.9 | 6.1 / 5.3 / 5.2 | 3 | +32.50% |
| Prepared make test | 124.7 / failed / 176.5 | 183.9 / 171.4 / 152.7 | 2 | +16.99% |
| Prepared make spec | 26.1 / 28.2 / 18.7 | 34.0 / 23.8 / 24.4 | 3 | +30.27% |
| Direct main-gate | 409.044158 / 326.320942 / 305.724082 | 526.893092 / 369.147291 / 391.294815 | 3 | +27.99% |
| Release build | 365.1 / 124.4 / 119.3 | 350.3 / 133.1 / 139.4 | 3 | +6.99% |

All six main-gate commands and release builds passed. Completed old prepared tests each report 4,373 Rust tests passed, 34 skipped, and 1,281 Python tests passed, nine skipped. Each candidate prepared test reports 4,279 Rust tests passed, 34 skipped, and 1,575 Python tests passed, ten skipped. The gate workloads therefore include different release test inventories. They are not equal-workload unit-test throughput measurements.

The same selected 16 gene-render tests execute in approximately 0.034 seconds in both second arms; retained Cargo output reports about 37.97 seconds for old test-profile compilation versus 57.08 seconds for candidate. This localizes the repeated gene edit-test cost to compilation/linking rather than execution of those selected tests. It does not identify a particular source change or explain every gate/spec cost. Shared load, graph changes and compiler-cache history remain limitations. The PM accepted these developer costs and assigned 2284; no additional probe or repetition was run to produce a causal claim.

| Size row, bytes except RSS | Old samples | Candidate samples | Comparable pairs | Paired change |
| --- | --- | --- | ---: | ---: |
| Target after completed make test | 2,766,790,656 / unavailable / 2,560,311,296 | 2,794,741,760 / unmatched / 2,582,937,600 | 2 | +0.95% |
| Target after release | 6,248,169,472 / 5,818,441,728 / 5,815,111,680 | 6,316,969,984 / 5,857,861,632 / 5,857,865,728 | 3 | +0.74% |
| Release peak RSS, KiB | 4,363,472 / 4,372,768 / 4,368,208 | 4,442,432 / 4,418,752 / 4,433,664 | 3 | +1.50% |
| Stripped executable | 33,916,016 each | 35,709,248 each | 3 | +5.29% |

The raw second-old after-test snapshot is 2,563,637,248 bytes and second-candidate is 2,614,403,072 bytes. The old gate failed, so these remain retained snapshots rather than a complete passing-test pair. Release executable digests are old `6874eb281b0d934cdbc40d8b1bf7de62fb6df43d319cd1ad2fb28b8777f3a11f` and candidate `8a95f7a2c7c0e542ac7a0d083df46b296cdfd6e3e5d33fdf009005f5b76326d0`.

## Runtime observations

Times are median seconds; ranges retain every measured sample. Change again uses the median matched ratio. `infinite` and `off` are the existing `BIOMCP_CACHE_MODE` values; default leaves it unset.

| Command/cache row | n pairs | Old median [range] | Candidate median [range] | Paired change |
| --- | ---: | --- | --- | ---: |
| --version, empty root | 5 | 0.007569 [0.005885–0.233658] | 0.007387 [0.006705–0.011276] | -1.76% |
| get gene BRAF, infinite | 5 | 0.188616 [0.183652–0.198180] | 0.185434 [0.184173–0.199895] | -2.01% |
| get gene BRAF, off | 5 | 0.444387 [0.429459–0.852987] | 0.441203 [0.421352–0.455087] | -0.72% |
| get variant 'BRAF V600E', infinite | 5 | 0.018127 [0.016114–0.023850] | 0.023930 [0.013078–0.032809] | +21.29% |
| get variant 'BRAF V600E', off | 5 | 0.498157 [0.489204–1.619128] | 0.510230 [0.472768–10.757967] | +2.57% |
| get drug imatinib, infinite | 10 | 0.243984 [0.231457–0.256038] | 0.234261 [0.222178–0.252090] | -4.00% |
| get drug imatinib, off | 10 | 0.910665 [0.881350–1.890054] | 0.922733 [0.872827–1.373725] | -0.35% |
| search article -g BRAF --limit 10, infinite | 5 | 1.383036 [1.351610–1.502981] | 1.437242 [1.415375–1.499320] | +6.23% |
| search article -g BRAF --limit 10, off | 5 | 1.933505 [1.864840–5.097419] | 1.916399 [1.852538–1.991807] | -4.19% |
| --json search all --gene BRAF --disease melanoma --counts-only, infinite | 5 | 1.034587 [0.981303–3.450964] | 1.040984 [0.997454–1.825343] | -2.09% |
| Same search-all command, off | 5 | 7.602686 [7.246845–12.013687] | 7.943771 [6.655063–9.409088] | +9.62% |
| --json search gwas -g BRAF, day/default | 5 | 0.007359 [0.006632–0.008047] | 0.008309 [0.007922–0.014245] | +16.50% |
| get drug imatinib, day/default | 5 | 0.230587 [0.226999–0.246449] | 0.230512 [0.225125–0.254638] | -0.03% |
| --version, populated workload tree | 5 | 0.006423 [0.006309–0.011532] | 0.006989 [0.006732–0.007534] | +6.51% |

All 174 runtime/prefetch processes exited zero and none reached the outer watchdog. This does not mean every provider succeeded: old cache-off search-all pair 61 reports `gwas search timed out after 12s`. Preserve that partial response and the candidate cache-off variant 10.757967-second outlier. Do not remove them or infer a cause from elapsed time. No separately timed MCP-startup row exists in the reused helper. The populated-tree version row uses the dedicated populated workload roots; `--version` does not initialize the database or MCP server. This is not a large-database startup, migration, or MCP-startup benchmark.

The two accepted watch rows have these raw old/candidate seconds, in chronological pair order. First-arm order alternates candidate/old, old/candidate, candidate/old, old/candidate, candidate/old in both rows.

| Pair | Warm variant old / candidate | Pair | Day-GWAS old / candidate |
| ---: | --- | ---: | --- |
| 17 | 0.018126707990 / 0.013077707990 | 71 | 0.008047083975 / 0.014245416998 |
| 19 | 0.016507249995 / 0.017536499974 | 72 | 0.007616291987 / 0.008414790995 |
| 21 | 0.023850166996 / 0.028928665997 | 73 | 0.007359083014 / 0.008309249999 |
| 23 | 0.016113583028 / 0.023929999996 | 74 | 0.006632083998 / 0.007922417019 |
| 25 | 0.021770583000 / 0.032808791002 | 75 | 0.007057791983 / 0.008222249977 |

Median paired increases are 5.078499 ms and 1.164458 ms respectively. All ten warm-variant stdout files are identical at 1,371 bytes, SHA256 `0746f675385674010d7c1d539ff29ded13a50be473f49efd81b73087496a890c`. All ten day-GWAS stdout files are identical at 4,930 bytes, SHA256 `e1dce06839998f8a068251486c82f4e7fd30e99ab16f9b1ec1d291e26118f33b`. Gene warm/off and variant off also match exactly in all five pairs. Drug, article, search-all and day-drug output does not match byte-for-byte across releases; this experiment does not assert equivalent returned data for those rows. It did not normalize output or interpret clinical content.

## Populated-cache accounting

Each row is one independently populated root per arm, captured after its processes closed. File bytes sum apparent file sizes; allocated bytes sum `st_blocks * 512`. Reported payload bytes are `cache stats`'s `blob_bytes`, which includes inline bodies in the candidate. Equal entry counts do not establish equal payloads or equivalent answers.

| Root | File bytes old / candidate | Allocated bytes old / candidate | Entries old / candidate | Reported payload bytes old / candidate |
| --- | --- | --- | --- | --- |
| Gene warm | 8,366 / 57,344 | 32,768 / 57,344 | 1 / 1 | 6,535 / 6,402 |
| GWAS day | 12,538 / 65,536 | 32,768 / 65,536 | 1 / 1 | 12,148 / 12,178 |
| Article warm | 65,479 / 122,880 | 110,592 / 180,224 | 5 / 5 | 63,071 / 63,023 |
| Variant warm | 455,018 / 503,808 | 475,136 / 503,808 | 1 / 1 | 452,401 / 452,268 |
| Drug warm | 1,754,411 / 1,805,381 | 1,798,144 / 1,855,488 | 4 / 4 | 1,747,775 / 1,747,182 |
| Drug day | 1,752,369 / 1,805,381 | 1,798,144 / 1,867,776 | 4 / 4 | 1,747,773 / 1,747,181 |
| Search-all warm | 2,648,152 / 2,707,143 | 2,740,224 / 2,723,840 | 14 / 14 | 2,641,526 / 2,639,603 |

Candidate SQLite file bytes in table order are 57,344; 65,536; 122,880; 503,808; 90,112; 90,112; 360,448. Closed sidecar bytes are zero for every candidate root. External blob bytes are zero for the first four roots, then 1,715,269; 1,715,269; 2,346,695. The article SQLite file occupies 180,224 allocated bytes despite its 122,880 apparent bytes. Old body files use a different layout, including `http/content-v2`; zero candidate external blobs in a small root does not mean zero stored response data. The observed SQLite layout adds tens of kilobytes for small roots, especially visible at one entry. PM accepted this storage trade-off. These seven small workload roots do not estimate a personal cache, large migration, long-lived database, or maximum disk budget.

## Failures, continuations and shared load

Initial admitted job `1792451426128-81984` stopped before samples because isolation omitted the standard sccache socket environment setting. Restoring that setting fixed preparation without changing compiler/profile/helper bytes. Job `1792451469191-87202` retained valid first-old build/edit rows, then the old prepared test failed its known default disk-floor cache assertion. Available disk was 182,852,767,744 bytes versus a 199,516,573,696-byte default floor. This is the 2220 fixture class; no old-release policy was modified. Disk proof `1792451856052-42901` and later headroom capture are retained. PM held collection until cleanup. At 297,889,263,616 available bytes, job `1792452855597-70577` resumed only missing first-old rows; make-test passed in 124.7 helper seconds. No valid earlier timing was repeated. Its wrapper later exited 143 for a deliberate coordinator boundary hold; completed first-arm commands exited zero. Bookkeeping/source proof `1792453884889-27409` preserved that arm.

Job `1792454239309-63056` collected both first and second candidate arms, then second-old prepared make-test failed `dropping_settlement_cancels_cacheable_article_provider_retry_and_admission` at `batch.rs:807`, left two versus right one. It ran 621 Rust tests successfully, one failed and 34 skipped; 3,751 selected tests were canceled. Python still passed 1,281 with nine skipped. The failed outer command elapsed 86.932429 seconds; it is not a completed make-test sample. The failure matches the exact assertion and preimage class documented in [2230](../tickets/2230-make-settlement-cancellation-proof-deterministic.md): paused Tokio time can auto-advance during real TCP readiness and allow a retry. Old `batch.rs` matches that diagnosed preimage. No new probe proved this individual occurrence's cause. The failed row remains incomplete, with no retry or substitution by its later passing main gate.

Job `1792456762741-28125` continued only naturally uncollected rows, completed the final pair, ran local candidate source-disable proof, then all runtime/cache observations. It passed in 2,345.646 seconds. A coordinator SIGSTOP/SIGCONT hold at the completed final-old boundary was outside timed rows. Both hold/resume markers are retained. The after-test size command was added at a natural continuation boundary without rerunning any arm. All three size snapshots exist, but only two follow passing old tests. Final archive job `1792459633640-65079` passed in 30.527 seconds and took no new samples.

This was a shared host, with recorded per-command and per-runtime load, and no quiet-window request. First candidate overlapped repair gate `2283/1792454342992-75134` and main full check `2283/1792454758788-87321`. A brief first-candidate evidence copy overlapped second-candidate release I/O. Final old overlapped lightweight diagnosis `2285/1792457442616-5675`. Runtime overlapped main full check `2285/1792458885162-87683`, which subsequently passed. Linux work used a separate host. These are load labels, not demonstrated causes of any timing change. No extra repetitions were collected to explain variance.

All regenerable measurement target directories were removed after arm evidence and release binaries were preserved. Source clones, environments and executables remain available for review and the coordinator's read-only consumer. Keep both raw experiments through their ticket/release consumers; defer their deletion until those consumers close.
