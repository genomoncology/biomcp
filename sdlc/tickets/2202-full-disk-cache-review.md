---
priority: 2
---

# 2202 — Review cache behavior on a full disk

Status: OPEN.

Milestone: 0.9.4

## Outcome

An agent test review fills a small dedicated filesystem during cache writes and eviction and checks requests still answer, no temp files leak and the size total stays right.

- Evidence: the 0.9.3 release review only simulated low disk with the free-disk setting; a real ENOSPC mid-write was not exercised.
- Proof: recorded results; file defects as tickets.

## Progress

- 2026-10-18 started
