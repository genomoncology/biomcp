---
priority: 1
---

# 2175 — Repair the HTTP fixture readiness failure

Status: COMPLETE.

Milestone: 0.9.3

Reviews: revision 28dc42b, accept

Reviews: revision 58e5e47, accept

Reviews: revision 45237a7, accept

Landed: 38610b4 (fast-forward: Accepted fixture readiness repair is already carried on main. Fifteen full main checks passed since startup hardening, latest 2156/1792285080324-84922 at 7025e1176195a9bfa6e4fcd8dbe84c30862cf1a6, with no readiness recurrence. Complete nonpublishing rehearsal https://github.com/genomoncology/biomcp/actions/runs/39085869827 passed on that candidate including qualification, packaged checks, all five archives and wheels, installed smokes and both containers. Original startup failure cause remains unconfirmed; accepted diagnostics and bounded readiness behavior passed observation. This closure changes ticket records only; tag the rehearsed candidate.)

## Outcome

The disable-source HTTP contract reliably starts its isolated server and proves that the flag and environment forms behave alike. Name and fix the actual readiness failure without weakening source-disable assertions or hiding child errors.

- Evidence: full main check 1792252542525-66782 on f56411687 failed serve_http_flag_matches_the_env_variable_form at tests/disable_sources_cli.rs:753. It reported that the child did not become ready at its loopback health URL within the fixture deadline. The assertion did not show child stderr or whether the child exited. Rust reported 4268 passes and one failure; Python reported 1166 passes. Current main differs only in ticket records.
- Starts from: the existing server startup/readiness helper and its child process handling.
- Changes: claims `tests/disable_sources_cli.rs` `crates/biomcp-mcp-contract-client/src/lib.rs` `CHANGELOG.md`; extend only to the existing shared fixture helper if the cause belongs there. A real runtime cause needs an explicit report before extending this fixture repair.
- Proof: the failed main job is the red evidence. Capture actual child state and error output in one focused build job, and use the smallest controlled check that distinguishes the named cause. Preserve flag/environment parity and no-provider-work assertions. Run affected tests through the build job, review, land the repair first, and run the full main check before other landings.
- Keeps: the source-disable contract, process cleanup, isolated configuration and cache state, bounded hung-process detection, and the routine slow-test warning policy.
- Defers: blind timeout increases, unrelated cache implementation, fixture framework changes and broad investigation. Report an unconfirmed cause honestly instead of retrying full gates.

## Progress

- 2026-10-17 started
- 2026-10-17 landed 38610b46596bf9099e58203a2442c6e5714a0562; next: The reviewed startup repair and diagnostic slice passed shared gate1792255317711-24775; full actual-main check runs now. Keep open for recurrence; runtime startup delay remains unexplained.
- 2026-10-17 next: PM permits closure after the final release rehearsal if no recurrence. Three clean full main checks since diagnostics landed:1792255511240-54847,1792256658840-2776,1792259305768-13311. Continue observing; runtime startup cause remains unconfirmed.
- 2026-10-17 next: Four clean full main checks since diagnostics landed; newest2156/1792260289066-54099 passed atbeb08a9. Keep observing through final release rehearsal, then close if no recurrence under PM ruling.
- 2026-10-17 next: Five full main checks passed since startup diagnostics landed; latest2156/1792263241297-93944 at22b4cecd2 passed after the separate MCP fixture correction. Continue observation through final release rehearsal; startup issue did not recur.
- 2026-10-17 next: Six full main checks passed since startup hardening; latest54fb25e in2156/1792266562026-19174. Cache batch2172+2173 now landed at62cefadf9 and its mandatory full main check runs. Startup issue has not recurred; continue observation through final rehearsal.
- 2026-10-17 next: Seven full main checks passed since startup hardening, latest62cefadf9 in2156/1792267362041-77812 after2172+2173. Startup issue has not recurred. The unchanged adverse-event MCP contract also passed fifty repetitions. Continue observation through final rehearsal.
- 2026-10-17 next: Eight full main checks passed since startup hardening, latest234d2b32d in2156/1792269541418-30247 after the reviewed TCP fixture repair. Startup issue has not recurred. Continue observation through final rehearsal.
- 2026-10-17 next: Ten full main checks passed since startup hardening, latest518a0e689 in2156/1792275455029-64826 after the five research fixes. No startup recurrence. Observe through final rehearsal.

## Investigation result

Restated outcome: keep HTTP flag/environment parity, isolated configuration and bounded child cleanup, and name the actual startup cause before claiming a repair. The relevant implementation is the existing shared ContractHarness HTTP startup helper. The failed main job is the reproduction evidence; targeted builds on the same source did not reproduce. Diagnostic-only build job 1792253367827-49992 passed the parity case. Job 1792253413183-55423 passed both existing serve cases; flag and environment children became ready in about 30 milliseconds. No timeout, runtime or assertion changes were made. The original failure omitted child state and stderr, so its cause remains unconfirmed.

PM decision, 2026-10-17: land bounded diagnostics with the cause marked unconfirmed, run one full main check, and resume the accepted batch if it passes. Keep this ticket open for recurrence. If it recurs, captured child state determines the fix; a fixture deadline under load belongs to the routine warning policy, where time alone never fails. This partial landing does not claim to repair the original readiness failure.

Review proof: job 1792253716177-76612 ran the real CLI with an invalid startup flag against the unchanged helper and failed because its returned error omitted the child's stderr and last probe. With bounded failure diagnostics, job 1792253771707-80949 passed that case and both existing serve parity cases, formatting and targeted Clippy. The helper reads at most 8192 stderr bytes, reports read failures without replacing the startup error, and retains awaited child kill on readiness exhaustion plus kill-on-drop cleanup. Successful startup adds no output or logging override. The original readiness cause remains unconfirmed.

The configured landing gate 1792253819443-83966 reproduced the HTTP parity failure and the new invalid-argument startup case. Both children were alive, with empty stderr and failed health connections, at the five-second helper deadline. The invalid-argument process ordinarily exits immediately; this establishes that the fixture can kill a live process before its argument parsing completes. It does not establish the underlying scheduler/runtime cause. The repair outcome is to treat that short latency threshold as a warning, preserve real child-exit errors and source-disable assertions, and retain separate bounded hung-process protection. The shared ContractHarness helper and its existing HTTP command tests are the implementation and outside-in test sites. A controlled delayed real CLI start must fail on the old helper and pass after this correction.

Controlled delayed-start proof 1792254207305-32303 failed on the old helper: a real CLI delayed six seconds before execution was still alive when the five-second fixture threshold killed it. Focused job 1792254230316-33422 passed the delayed healthy and invalid-argument cases plus both existing serve cases, formatting and targeted compiler lint after the fix. Job 1792254268688-35247 passed full lint and the quality ratchet. Fresh review accepted source af1b8022e without changes. The helper now warns once after five seconds and waits for readiness or an actual child exit; a separate 120-second hung-child watchdog and one-second probe timeout preserve bounded cleanup for callers outside the test runner. This fixes the premature fixture kill, while the original operating-system/runtime startup delay remains unknown. Keep this ticket open for recurrence under the PM decision above.

PM confirmation, 2026-10-17: warn at five seconds while the child remains alive, fail on child exit, and retain an outer hung-process watchdog. The PM identified first-launch operating-system scanning as one possible explanation for delay before argument parsing; this is unverified and not the basis of the fixture correction. No production startup change is justified by the available evidence.
