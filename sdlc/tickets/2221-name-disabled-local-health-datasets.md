---
priority: 2
---

# 2221 — Name disabled local health datasets

Status: COMPLETE.

Milestone: 0.9.4

Reviews: revision f92aad4, accept

## Outcome

Health names the disable setting for every disabled local dataset, consistently with the existing DDInter row, without calling a configured disable a dataset error.

- Evidence:2204 Linux review at3dc15da, cases0225/0226, disabled all81 registry IDs by environment and flag. Both made zero traced requests but reported EMA,CVX,WHO-PQ,GTR andWHO-IVD as status:error without disabled_reason; error count5. Existing DDInter correctly reports disabled.
- Changes: `tests/test_disable_sources_replay_walk.py` `src/cli/health/local.rs` `CHANGELOG.md`. Use the existing disabled-source row handling across the five local checks; declare the existing outside-in test path before editing it.
- Proof: reproduce `BIOMCP_DISABLE_SOURCES=$(jq -r 'map(.id)|join(",")' docs/reference/sources.json) biomcp --json health` with isolated fixture state. Check both environment and flag origins, exact disabled reason for the five rows, and zero provider requests; retain the enabled/local-data behavior. Raw review observations are described in2204's record.

## Progress

- 2026-10-18 started

Pre-code plan: restate the outcome as truthful disabled local-health rows for both flag and environment origins, preserving enabled behavior. Add two real-CLI cases in the existing replay walk using shared isolated environment and request tracing; all six local rows must name the disable setting, report no missing files or errors, and issue no replay requests. Reuse the exact current87d53c batch CLI (verified banner0.9.4-dev+g87d53c67, SHA25631df21871fbf9ca95580f2ac15a05d3bb8f53079672db177e5363b6a9d611a8f) for focused RED. Only after RED, share the existing DDInter disabled outcome construction across all six local checks.

Actual current-main RED: job2221/1792355146000-16702 reused verified87d53c artifact and ran the two disabled_local_health origin cases in tests/test_disable_sources_replay_walk.py. Both failed as expected because EMA,CVX,WHO-PQ,GTR,WHO-IVD reported status:error while DDInter reported disabled;2failed,2deselected in0.37s. The first fresh-venv invocation lacked pytest; the corrective invocation reused the existing prepared2220 Python environment without rebuilding. Proceed with the declared shared local-health outcome helper and focused GREEN.
- 2026-10-18 next: Ready for fresh review at sourceee3311ee33924677bdd06cf1c67d33939c62c4c3. Current-main RED1792355146000-16702 reproduced both origins. GREEN1792355391569-49900 passed2realCLI cases,38existing local-health Rust cases and make lint including warnings-denied Clippy,format,quality/boundary checks. BinarySHA256442a92283e4892499d21507aa7e7c4e02f5ee7d1299bd44dd094bd2a67a9d2a5 was built from87d53c plus matching patch. One test-only correction snapshots request count after replay startup; no behavior assertion weakened. Queue with2222 for the approved batch.
