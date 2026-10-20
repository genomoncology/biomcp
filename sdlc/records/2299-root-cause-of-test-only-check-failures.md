# 2299 — Root cause of test-only check failures, 17 to 20 October 2026

A read-only analysis following `skills/rca.md`, run for the PM on 2026-10-20. It changed nothing. Sources were the tickets and records on main, the 2294 and 2298 branches, and the PM and coordinator mail.

## Cases

About 30 failed landing gates, full main checks and qualification runs found no product bug. In the same period those runs caught four or five real bugs: 2208, 2242, 2224, 2286 and part of 2248. Coverage, live capture and review found the rest (2287, 2292, 2296, 2203, 2206, 2256).

| Group | Cases | Category |
|---|---|---|
| Best-effort concurrency | 2225, 2275, 2283, 2285 | The test expected every concurrent write to land; answers were correct |
| Fake time with real IO | 2230, 2276 | Paused Tokio clock advanced while real TCP was pending |
| Fixed deadlines | 2298, 2178, 2175 | A correct program missed a short deadline when the host stalled |
| Fixture lifecycle | 2294 fixture, 2229, 2260, 2257 | Ad hoc servers panicked, leaked, or copied harness files lacked dependencies |
| Host environment | 2186, 2187, 2188, 2207, 2209, 2176, 2220, 2255 | Host Cargo, Git history, HOME, live calls or disk state leaked into tests |
| Build cache and job environment | 2297 and one incomplete job environment | Retained wheel target or an incomplete environment |
| Repo-wide checks run late | 2262 (twice), 2161, 2196, 2265 | Consistency tests first met the change at the shared gate |
| Wrong tool | 2281 | The live checker sent different headers from the product |
| Unclassified | 2174 | Cause never established |

Four lanes (2275, 2283, 2285, 2298) ended with the original cause unknown.

## Root cause

Tests assert things that hold only on a quiet, isolated host: every write lands, a deadline is met, a paused clock stays put, the host's tools are absent, the disk has room. Focused proofs run on an idle machine, so the assertions pass there. Full checks run beside other jobs and fail. Shared test support offers no ready way to reuse admitted answers, wait on an event, own the environment or serve a socket safely, so each test builds its own. Teardown deletes the failed case's state and no host load is recorded, so each failure is diagnosed blind and ends in a full rerun. Sibling audits are manual reads, so they miss cases.

## Fixes

Filed in 0.9.4, smallest first: 2299 keep failed-case state; 2300 shared best-effort reuse helper; 2301 finish the paused-clock repairs and forbid the mix; 2302 event waits instead of fixed deadlines; 2303 one shared Rust test HTTP server; 2304 contract tier in the local command.

Already done: 2288, 2289, 2290, 2291, 2297, 2229, 2260, 2257, 2220, 2186, 2187, 2188, 2209.

Sent to Jarvis: pm job capturing host load and co-running jobs on a test failure; build disk floors; incomplete job environments; container build targets reused across runs; four habits for `skills/outside-in-tests.md` (wait on events, not clocks; never mix paused clocks with real IO; assert the contract, not best-effort side effects; keep a failed case's state).

## Severity

Medium. It repeatedly held main and release admission and wasted build-machine time. No wrong answer reached a clinician, and the real bugs were still caught.
