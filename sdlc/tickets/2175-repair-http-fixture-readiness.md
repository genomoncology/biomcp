---
priority: 1
---

# 2175 — Repair the HTTP fixture readiness failure

Status: OPEN.

Milestone: 0.9.3

## Outcome

The disable-source HTTP contract reliably starts its isolated server and proves that the flag and environment forms behave alike. Name and fix the actual readiness failure without weakening source-disable assertions or hiding child errors.

- Evidence: full main check 1792252542525-66782 on f56411687 failed serve_http_flag_matches_the_env_variable_form at tests/disable_sources_cli.rs:753. It reported that the child did not become ready at its loopback health URL within the fixture deadline. The assertion did not show child stderr or whether the child exited. Rust reported 4268 passes and one failure; Python reported 1166 passes. Current main differs only in ticket records.
- Starts from: the existing server startup/readiness helper and its child process handling.
- Changes: claims `tests/disable_sources_cli.rs` `CHANGELOG.md`; extend only to the existing shared fixture helper if the cause belongs there. A real runtime cause needs an explicit report before extending this fixture repair.
- Proof: the failed main job is the red evidence. Capture actual child state and error output in one focused build job, and use the smallest controlled check that distinguishes the named cause. Preserve flag/environment parity and no-provider-work assertions. Run affected tests through the build job, review, land the repair first, and run the full main check before other landings.
- Keeps: the source-disable contract, process cleanup, isolated configuration and cache state, bounded hung-process detection, and the routine slow-test warning policy.
- Defers: blind timeout increases, unrelated cache implementation, fixture framework changes and broad investigation. Report an unconfirmed cause honestly instead of retrying full gates.
