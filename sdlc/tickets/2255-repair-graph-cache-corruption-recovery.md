---
priority: 1
---

# 2255 — Repair graph-cache corruption recovery

Status: OPEN.

Milestone: 0.9.4

## Outcome

Restore the graph-cache corruption contract on current main. A corrupt record must behave as a miss, recover through the owning provider route, and be replaced when the isolated fixture permits cache writes. Fix the demonstrated cause; preserve corruption handling, provider evidence and failure assertions.

Changes: `src/entities/article/graph/tests/cache.rs`, `CHANGELOG.md`, `sdlc/tickets/2255-repair-graph-cache-corruption-recovery.md`.

## Evidence

Actual-main job 2156/1792428576000-26345 at 3926961f2f2ac9f1b695561690d44cef7177f7ea ran `env -u CARGO_INCREMENTAL make main-gate`. The case `entities::article::graph::tests::cache::a_corrupt_record_is_a_miss_and_is_replaced` failed at cache.rs:506: expected 1, observed 0. The same source passed its preceding landing gate. Rust stopped early; the Python cases passed. The retained failing main run is the reproduction, not a claim that unrelated checks passed.

Keep: experiments/776-biomcp-2156-main-checks — retain the original failed main check while this repair is investigated.

## Keeps

Preserve the existing corruption and replacement assertions. Do not turn an intermittent failure into a retry-until-green test. Use the shared fixture isolation policy if configuration, disk or environment proves responsible. Hold ordinary landings only until the reviewed repair and resulting main pass.

## Proof

Name the exact cause, run the owning regression under the condition that exposed it, and run the affected cache controls. Run mechanical checks for changed source. Record raw evidence in the repair lane's numbered retained experiment; the coordinator reruns main after landing.

## Repair boundary

The failure occurs while asserting the first seed row, before corruption is inserted. Inspect the graph fixture against the shared scratch-cache policy and prove any inherited disk-floor cause under a controlled threshold on both prior main 569d9beb142c0ccb70e65d5f8618f5ee8d07a4d7 and current main. Reuse the shared fixture guard rather than copying its settings. Preserve every existing corruption and replacement assertion.

## Progress

- 2026-10-19 started

## Verification

Previous-main job 2255/1792429064983-51930 on 569d9beb142c0ccb70e65d5f8618f5ee8d07a4d7 reproduced the unchanged first seed-row assertion under inherited BIOMCP_CACHE_MIN_DISK_FREE=100%, normal cache mode and a 10485760-byte size limit. The repair reuses the central scratch-cache guard for the fixture lifetime instead of copying policy. Green job 2255/1792429189362-481 passed all twelve owning graph-cache controls under identical inherited pressure and mechanical lint. Existing corruption and replacement assertions are unchanged. The committed source matches the tested manifest and patch. No provider lookup or routine/main gate was repeated by the lane.

Keep: experiments/800-biomcp-2255-cache-retention — retain original-main failure, controlled previous-main red, commands, checked patch, source hashes and green raw results.
