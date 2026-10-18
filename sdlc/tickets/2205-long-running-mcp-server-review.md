---
priority: 3
---

# 2205 — Review a long-running MCP server

Status: OPEN.

Milestone: 0.9.4

## Outcome

An agent test review runs `biomcp serve` for several hours under steady load across the daily clean and evictions, and checks answers, memory growth and cache accounting.

- Evidence: the 0.9.3 release review ran short MCP sessions only.
- Changes: `sdlc/records/2205-long-running-mcp-server-review.md` `sdlc/tickets/2205-long-running-mcp-server-review.md`. Review/report only; file product defects separately.
- Proof: recorded results; file defects as tickets.

## Progress

- 2026-10-18 started

Pre-execution scope: prepare a bounded actual long-running stdio MCP experiment with recorded loopback replies, controlled small cache/eviction pressure, response checks and process memory snapshots. Reuse existing fixture/client helpers and qualified artifact. Name any controlled due-marker event separately from actual elapsed runtime; no clock framework/product hook. Report the finite load/resource/stop plan before coordinator admission; no product patches in this review.
