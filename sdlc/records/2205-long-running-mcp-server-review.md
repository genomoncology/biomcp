# Long-running MCP cache review

One initialized stdio MCP server answered correctly for 10,801.080 actual seconds under a bounded, steady workload. Cache eviction and a controlled daily-clean event preserved answers and byte accounting. The review found no product failure in this workload; it does not establish that memory cannot grow over a longer run.

## Artifact and workload

The executable came from the passing full main check of `8911f0e2cdefa702384b09d282e41695056da239`, without another build. Its banner was `biomcp 0.9.4-dev+g8911f0e2`; SHA-256 was `75c97753e4ab890c3d5714ab128f7f4fa98988024434d927ef3ca931ac3d0579`. Build job `2205/1792357841395-14251` passed with exit 0 in 10,802.554 seconds overall, beginning on October 18, 2026 at 21:10:43 UTC. The owned experiment is `2078`.

The one-shot driver reused `RawStdioMcp` and `retained_cache_env`, with an ordinary `initialize` request and `notifications/initialized`. One server PID remained alive for the whole timed run. Every five seconds it received `tools/call` for `biomcp --json search gene BRAF<N> --limit 1`, alternating a new request and an immediate repeat over a 16-key ring. Each reply had to contain the same BRAF/673 result and result digest.

A loopback HTTP fixture served the existing `testdata/sources/mygene/search_braf_20260811.json` response, preserving its gene fields and adding an ignored `_cache_review_key` containing the request path. This made the content blobs distinct: identical reply bytes would deduplicate and provide no size pressure. The fixture had a one-day HTTP freshness lifetime; the isolated cache budget was 32,768 bytes with a one-byte free-space floor. No live provider, credentials, or production cache was used.

## Observations

- All 2,160 scheduled replies passed. The 1,080 immediate warm reads made no provider requests; all 1,080 cold reads reached the fixture. The controlled cleanup's separate CLI read brought the provider total to 1,081. Two additional MCP requests checked the controlled event.
- Across 181 settled snapshots, actual content never exceeded 31,232 bytes. Every saved `v2` byte total equaled the sampled physical content total; no undercount or conservative excess was observed. Successive samples showed 717 content-path disappearances, demonstrating repeated eviction. The final cache contained four files totaling 31,208 bytes, with marker `v2 31208`.
- Process RSS hourly medians were 43,696, 43,888, and 44,096 KiB; the final sample was 44,256 KiB. This modest upward drift, approximately 400 KiB between the first and third hourly medians, does not establish a leak or demonstrate a plateau. The initial-to-final RSS range was 40,272–44,256 KiB. Final accumulated process CPU time was 2:08.01.
- Host load was shared with other jobs: sampled one-minute load ranged from 0.695 to 12.528. Request latency had median 0.0154 seconds and maximum 0.2832 seconds. These are workload observations, not a quiet-host performance comparison.

At elapsed 5,400.126 seconds, the driver aged only its owned `.last-startup-clean` marker to more than a day old. A further request to the already initialized MCP middleware left that marker unchanged. One ordinary CLI request against the same cache then refreshed the due marker. The same MCP PID answered correctly afterward and continued through the remaining actual runtime. This proves coexistence with another process's due startup cleanup. It does not prove an automatic recurring cleanup inside an initialized MCP server or actual 24-hour expiry; middleware initialization is retained in `SharedHttpCache::ready`, and the due check belongs to manager construction.

## Setup corrections, cleanup, and limits

Two short setup attempts preceded the completed run and provide no long-duration evidence. Job `2205/1792357689344-1515` stopped after one valid reply because the driver mistakenly counted its retained executable toward the log ceiling. Job `2205/1792357717973-3024` stopped after five seconds because the reused pre-session client had not completed initialization; the server emitted the existing initialization guidance. The coordinator approved the narrow corrections to count evidence files only and perform the existing initialization exchange. Neither changed product code. Both attempts were preserved separately for review.

The completed run used approximately 584 KiB of evidence logs and stayed below its protective 512 MiB RSS, 64 MiB content, and 20 MiB log ceilings. Those ceilings protect the experiment; they are not product guarantees. After the successful final snapshot, the driver terminated and reaped its owned server and stopped its fixture threads. Independent review checked the driver, plan, retained executable hashes, representative replies, request observations, resource snapshots, and both setup attempts against this record. After acceptance, ownership and all three finished jobs were verified, and no live owned process or other consumer remained. The authorized cleanup removed experiment `2078`; build-job logs remain retained.

This review covers one process, one recorded provider response, serial fixed-rate requests, repeated size eviction, and a controlled cleanup from another process on macOS. It does not cover arbitrary providers, parallel request saturation, a full-day wall-clock crossing, power loss, exhaustive schedules, or unbounded memory growth. Existing Windows concurrency and real full-disk results remain separate evidence. No additional product requirement or repair follows from this bounded run.
