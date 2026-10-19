---
priority: 3
---

# 2257 — Route every test cache root through the shared guard

Status: OPEN.

Milestone: 0.9.4

## Outcome

No test creates its own cache directory outside the shared fixture guard, so every test gets the same disk-floor and cache policy, and a check refuses new ones.

## Evidence

The same failure has occurred twice: 2220 and 2255 each traced an intermittent failure on a nearly full build machine to a test fixture with its own cache root that bypassed the shared policy pinning `BIOMCP_CACHE_MIN_DISK_FREE`. Each repair fixed one fixture.

## Keeps

Change test fixtures only; no product change. Keep every test assertion. No repository-wide test refactor beyond moving cache roots onto the shared guard.

## Proof

An inventory lists every test that sets a cache directory and shows each uses the shared guard after the change. A check in the existing quality checks fails on a new test that sets a cache directory without it.
