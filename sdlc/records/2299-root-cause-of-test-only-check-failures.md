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

## Failed replay state ownership implementation

The implementation starts from seed `3819a97abb142b7d57f71eada2e8a4031b9c65a9`, containing main `91118b5881ccbf4186875e445173b61ec4dbc86d`. The new outside-in check starts a real nested pytest runner, executes the CLI against replay with a retained cache, deliberately fails one case and passes another, then reads the failure's SQLite entries and request log after the nested runner and shared server exit. Before implementation, job `2299/1792523971430-33961` failed that check with `AssertionError: failed replay command state was deleted` and `1 failed in 1.76s`. This reproduces the evidence-loss mechanism; it does not recover the original runtime causes listed above.

The pytest case fixture lends its replay subdirectory to the replay session. Each borrowed view puts its command directories there and snapshots only its request-log slice before restoring the shared fixture root. The final implementation keeps pytest storage through teardown, then removes passing replay subdirectories after the final test report. Other pytest temporary files retain the default retention behavior. Standalone replay callers keep their temporary lifetime and cleanup. Cache admission, cache policy and runtime code do not change.

Job `2299/1792524044897-40885` passed 46 affected cases across replay ownership, server lifetime, fail-soft cache, SQLite, persistent size and search-all provider failure; it also passed the golden provider-override case and changed-file Ruff. Its private driver then failed because it named the tracked-text checker with an incorrect `.py` suffix. This was a command error and invalidates no completed test result.

Evidence remains in canonical experiment `2102-biomcp-2258-final-live-replay/failure-state/2299`, with an OWNER and Keep instruction. The immutable CLI copy matches the supplied current-main debug binary at SHA256 `338e94737bb9bdd5f4d72bd0d95e7adec50b34d3271feae6b920c71dab83b2ba`; `artifact.json` records its source revision and source/copy identity. The source archive and command JSON files retain the exact checked input and command lines. These focused ownership checks did not build native code, run a full gate, call live providers or qualify Linux offline behavior.

Final job `2299/1792524164427-89717` passed the strengthened ownership control, changed-file Ruff, tracked-text checks, quality ratchet and staged/unstaged whitespace checks. The control also requires the failed command directory to sit below the nested runner's pytest base. Its nested runner uses the repository's failed-only policy. The outer proof alone uses `-o tmp_path_retention_policy=all` so its passing assertion does not erase the deliberately failed nested case. The retained `retained-control` tree contains the nested failure's actual `cache/biomcp.db`, per-view `request.log`, runner output and passing/failing path markers; the passing nested view is absent. `final-commands.json` and `final-checked-inputs.json` identify this run. The final regression file SHA256 is `8274e0959fe7debcc850d52f31e5223fb2a28f26c841c3a7ecc9c26864f36349`. The other checked code/configuration inputs match the earlier 47 passing affected cases.

## Routine gate correction

Job `2299/1792527310510-12695` ran `make landing-gate` on clean committed candidate `d5a945990f1b9bec2b05ab4edbce9f04a5b9bc42`, which includes repaired main `7503f6fd9927d86d8cecb3c260fec893c2380f32`. A retained Git bundle supplies that candidate to the job. The job prepares its own Python environment, uses its own built CLI and sets the ticket branch identity for changelog checks. Lint and the Rust lane passed. The Python lane reported 1731 passed, 11 skipped and one failure: the binary-module contract requires a module-level `pytestmark`, while the new regression used a function decorator. Specifications did not run because the test phase failed.

Commit `e912d76352ef590f6c2b34029ebec2a36e44c260` moves the existing `needs_binary` marker to module scope. The regression behavior does not change. Original logs, the clean-input result and the failed pytest state remain in `failure-state/2299/routine-d5a945990` under the canonical experiment. The next routine runner checks the affected marker contract before running the required landing gate.

The corrected routine passed in M5 job 2299/1792528132354-6729 on clean committed candidate 851e73b534ed7ff98af0fc2e216895b20970187e, with actual repaired main 7503f6fd as ancestor and the ticket branch identity set. The affected module-marker contract passed first, followed by the full landing gate. The candidate was pushed once after that pass. Complete raw logs, results and exact source remain in failure-state/2299/routine-851e73b53 and candidate-851e73b53.bundle. Original failed-case data remains separately under routine-d5a945990. Darwin provided no network sandbox. This records routine correctness, not Linux offline qualification.


## Reviewer failure-phase repair

The PM analysis above was imported as planning context. It does not establish the original runtime cause of the four historical failures. The replay controls establish evidence loss and preservation only.

Gator job `2299/1792528707965-1252032` checked the committed implementation overlay on published main `7503f6fd9927d86d8cecb3c260fec893c2380f32` in its own Python environment. A real nested pytest runner created command state and a replay request log, then failed during setup, call or teardown. The call failure retained both artifacts. Setup and teardown failures deleted both. Pytest 9.0.3 removes `tmp_path` when the call passed or did not run, before the teardown result exists. The earlier call-only proof did not cover these failure phases.

The repair selects pytest's default `all` temporary retention and removes only each passing replay subdirectory after its teardown report. A failed setup, call or fixture teardown keeps the replay subdirectory. This also removes the earlier implementation's broad deletion of every passing pytest temporary directory. Other temporary files keep main's default behavior. Each worker owns its pytest session and case directory; each borrowed replay view owns a unique directory and request-log slice. Standalone replay ownership still cleans up on exit.

Retention applies to normally reported pytest failures after case ownership starts. A killed runner cannot promise a completed request-log snapshot. Pytest's retention count can expire older run directories, and an explicit reused `--basetemp` can delete an older run. Durable diagnosis requires copying the failed case before those cleanups.

The expanded repository regression covers setup, call and teardown errors with the prepared CLI. Gator job `2299/1792528804184-1252449` passed the three lightweight phase controls and 13 affected tests under two xdist workers, including the expanded regression and replay lifetime suite. That job used a controlled Python command to write a SQLite entry and request a recorded replay route; it did not qualify the real CLI. Its driver then failed because Ruff was absent from the host PATH. The completed test results remain valid. A mechanical-only retry passed Ruff, then failed because the tracked-text checker was invoked directly without executable permission. The corrected driver invokes it with Python. Red controls, positive controls, exact source overlays, fake command and raw logs remain under `failure-state/2299/reviewer-phases` and `reviewer-phases-green` on Gator. The original M5 binary and earlier evidence remain untouched.

The earlier routine pass on `851e73b53` qualifies its original call-only implementation. The shared teardown hook and expanded regression change executable inputs and require a new routine check.


Gator mechanical job `2299/1792528879304-1253965` passed changed-file Ruff and the tracked-text scans. The quality ratchet reported a tooling error because `mustmatch` was unavailable on Gator. No native compilation ran. Its exact source overlays, commands and output remain under `failure-state/2299/reviewer-mechanical-final`. This run does not claim a quality-ratchet pass. The required routine check must supply that gate. Local whitespace validation passed for the repair.


## Reviewed routine qualification

M5 job `2299/1792529260237-46688` passed on clean committed candidate `a141519870ed8c8a6bb174f2e40f5dcc4d8be882`, containing reviewed published main `267f583c2a28605b0a4998a0d282812fbd7a132d`. The retained runner fetched the exact candidate bundle, asserted that revision and published-base ancestry, set the ticket branch identity, prepared its own Python environment and ran the binary-module marker contract followed by `make landing-gate`. Every command exited zero and the final tracked-input check was clean. The expanded setup, call and teardown regressions passed against the job's real CLI under the routine worker run. This routine supplies the quality-ratchet gate that Gator could not run. It supersedes the earlier routine qualification for the changed retention implementation.

Complete source and raw results remain on M5 under `failure-state/2299/candidate-a14151987.bundle`, `landing_gate_review_a14151987.py` and `routine-a14151987`. The final evidence commit changes prose only. The gate proves routine correctness on Darwin; Darwin supplied no network isolation, and this run makes no Linux offline claim. Original red phase evidence, positive retained state and the original immutable CLI remain untouched.
