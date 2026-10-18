---
priority: 3
---

# 2205 — Review a long-running MCP server

Status: OPEN.

Milestone: 0.9.4

## Outcome

An agent test review runs `biomcp serve` for several hours under steady load across the daily clean and evictions, and checks answers, memory growth and cache accounting.

- Evidence: the 0.9.3 release review ran short MCP sessions only.
- Proof: recorded results; file defects as tickets.
