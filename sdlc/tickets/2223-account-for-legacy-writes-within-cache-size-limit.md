---
priority: 2
---

# 2223 — Account for legacy writes within the cache size limit

Status: OPEN.

Milestone: 0.9.4

Depends on: 2218

## Outcome

A current command keeps the configured HTTP cache size limit after an older released binary writes to the same cache without updating the current saved byte marker. Keep startup free of a full tree walk and preserve provider fail-open behavior and request deadlines.

- Evidence:2203 job1792355882950-20969 used exact87d53c current and verified publishedv0.9.2 with a50000-byte limit. Currentseed left17440actual/17440marker; oldwrite left34882actual/17440marker; nextcurrentcommand returned validBRAF but left52332actual/34890marker, exceeding the limit. Daily marker remained fresh. This is more than an expected intermediate legacy stale marker.
- Changes: `src/cache/manager.rs` `tests/test_cache_persistent_size.py` `CHANGELOG.md`. Diagnose before editing, declare any additional exact paths before changing them, and reuse existing legacy-writer/cache fixtures without a new framework or runtime dependency.
- Proof: preserve2203 owned experiment2077 failingcache/results and verified binaries until consumed. Reproduce on currentmain, add a deterministic outside-in regression without requiring a public download in routine tests, fix the narrow cause and run affected checks. Retain existing fast-startup/deadline/provider behavior; do not introduce a full startup scan. After repair,2203 must finish both controlled daily-clean checks and prove exact marker reconciliation.

PM ruling: priority2 because mixed versions are uncommon, but the configured size limit must hold. Keep repair narrow and start-up free of a full tree walk.2203 remains open for the two daily-clean checks that stopped before execution.

## Progress

- 2026-10-18 started
