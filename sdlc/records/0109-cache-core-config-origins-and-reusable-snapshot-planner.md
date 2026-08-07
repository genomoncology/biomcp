---
base: 7705f44134b005512687dcc9e07d9dba8e94dc25
head: e08cb56c36711692b38417f7bf13f634dffb48d7
---
Tickets 095B, 095C, and 095D (cache CLI commands) all need access to the same cache inspection and cleanup planning logic, but that logic doesn't exist as a reusable internal module yet. This ticket adds the internal cache-core foundation that downstream CLI tickets call — no new user-visible commands.

Imported from March ticket 109. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/109-cache-core-config-origins-and-reusable-snapshot-planner
