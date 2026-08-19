---
flow: build
priority: 7
hold: draft for review; do not promote until Ian releases this
---
# Stop the Windows contract suite failing on a live network call

`windows_cache_epoch_files_are_user_only_and_reject_hard_links` in `tests/managed_state_permissions.rs` failed CI on 2026-08-19 with a Windows socket error while reading a MyGene request:

```
repaired MyGene fixture result: "read MyGene request: A non-blocking socket operation
could not be completed immediately. (os error 10035)"
```

The identical commit passed on re-run with no change to the tree, so the failure is intermittent rather than a real defect. That is the problem. A test in a deterministic gate is reaching a live external service, which is exactly what `sdlc/planning/verify-lane.md` argues a gate must never do: an unattended run must not be judged on someone else's uptime or on a transient socket condition.

The cost is not one red build. Five of the last twelve CI runs on `main` were red, and a branch that is red this often trains everyone to re-run rather than read, which is how a real failure gets missed. The test's actual subject — that cache files are user-only and reject hard links — has nothing to do with the network.

## Done when

- The test exercises the file permission and hard-link behaviour it is named for, without depending on a live network call succeeding.
- The test passes repeatedly on Windows without a re-run.
- Any genuinely live coverage this test was providing is either kept in the verify lane, where live failures are expected and read by a person, or is deliberately dropped with a note saying so.
- The design states whether other tests in the deterministic gates reach live services, since a single instance and a pattern call for different responses.
