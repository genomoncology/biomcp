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

Approved finite plan: one persistent stdio MCP process for10800seconds, fixed five-second request cadence and bounded16-key recorded gene-search ring on32KiBcache/minfree1B, with cold/warm answer/protocol checks, evictions and process RSS/CPU/cache accounting snapshots. Keep representative replies and bounded logs. At midpoint age only the owned daily marker, observe that initialized middleware does not recheck it, then have one ordinary CLI command perform due cleanup while the same MCP process keeps answering. This proves coexistence across another process daily clean, not a recurring in-server timer or actual24h expiry. Existing code initializes middleware once; no new product requirement follows from this observation. Stop on actual failure/protective ceilings or finite deadline, always reap fixture/server and write result. Preparation is approved; long job waits for explicit admission and frozen qualified artifact.
