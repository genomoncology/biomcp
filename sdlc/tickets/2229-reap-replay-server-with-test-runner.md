---
priority: 2
---

# 2229 — Reap the replay server when its test runner exits

Status: OPEN.

Milestone: 0.9.4

## Outcome

A replay server exits when the test run that started it dies, including a killed runner, timeout, or failed teardown. No server survives an abandoned test run.

- Evidence: an orphaned server survived a deleted 2114 checkout. A second server survived the closed 2077 gate (1792357738937-3997), was reparented to the system, and prevented cleanup of the finished build. Both required manual termination. These are recurrences of runner lifetime escaping test ownership.
- Starts from: existing replay-server launchers and teardown helpers. Inventory their ownership before choosing a small shared fix.
- Changes: `tools/replay-server*` and the existing test helpers that launch the replay server, plus their focused tests and the dependency in `sdlc/tickets/2111-move-the-remaining-fixture-consumers-onto-the-replay-server.md`. Narrow the exact paths after the initial inventory; do not migrate consumers here.
- Proof: reproduce an orphan on current main with an isolated runner; kill that runner midway through a replay request and verify that its server exits and its port closes. Cover normal teardown and the existing timeout/failure path without introducing a second replay framework.
- Keeps: replay routes, captures, request matching and product behavior remain unchanged.
- Order: land before 2111 moves more consumers. The PM approved this bounded prevention ticket after the second recurrence.

## Progress

- 2026-10-19 started
