---
ticket: 1285
head: 5f84e585
merge: (this record's merge)
---

# 1285 — the rehearsal found the DDInter smoke gap

The first release rehearsal (scratch run 36872650687, 2026-10-01) failed all five wheel-smoke legs: the smoke queried DDInter interactions in a clean venv, and the query path is local-dataset by design (a cold cache errors; only `biomcp ddinter sync` downloads the bundle). Reproduced locally with the run's own wheel: cold error, sync success, warm query answer. The provider itself was healthy (all eight CSVs answered 200 with the biomcp user agent). The fix runs `ddinter sync` in wheel-smoke before the query; the step pin is updated in the same commit. Code review: ACCEPT 2026-10-01 dispatch 69d30bbb-c876-4a8a-97d5-7e52dca0f51f. The release workflow had never run its tag-push path before this rehearsal; CI cannot reach it.
