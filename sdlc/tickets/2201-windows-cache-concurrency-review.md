---
priority: 2
---

# 2201 — Review Windows cache concurrency

Status: OPEN.

Milestone: 0.9.4

## Outcome

An agent test review on Windows covers parallel CLI runs, an MCP server and `cache clear` sharing one cache, and the size marker, under Windows' mandatory locks.

- Evidence: the 0.9.3 release review ran on Linux only; 2170's Windows CI case covers lock contention but not the full concurrent matrix.
- Proof: a recorded run on Windows with results; file defects as tickets.

## Progress

- 2026-10-18 started
