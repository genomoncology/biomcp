---
priority: 2
---

# 2229 — Reap the replay server when its test runner exits

Status: COMPLETE.

Milestone: 0.9.4

Reviews: revision a5a47e9, accept

Reviews: revision fcacae8, accept

## Outcome

A replay server exits when the test run that started it dies, including a killed runner, timeout, or failed teardown. No server survives an abandoned test run.

- Evidence: an orphaned server survived a deleted 2114 checkout. A second server survived the closed 2077 gate (1792357738937-3997), was reparented to the system, and prevented cleanup of the finished build. Both required manual termination. These are recurrences of runner lifetime escaping test ownership.
- Starts from: existing replay-server launchers and teardown helpers. Inventory their ownership before choosing a small shared fix.
- Changes: `tools/replay-server` `scripts/run-specs.sh` `tests/test_replay_server_lifetime.py` `tests/test_ci_workflow_contract.py` `CHANGELOG.md` and the recorded dependency in `sdlc/tickets/2111-move-the-remaining-fixture-consumers-onto-the-replay-server.md`. The Bash launcher needs only exec so its process-substitution shell cannot outlive the test runner and masquerade as the server owner.
- Proof: reproduce an orphan on current main with an isolated runner; kill that runner midway through a replay request and verify that its server exits and its port closes. Cover normal teardown and the existing timeout/failure path without introducing a second replay framework.
- Keeps: replay routes, captures, request matching and product behavior remain unchanged.
- Order: land before 2111 moves more consumers. The PM approved this bounded prevention ticket after the second recurrence.

## Development brief

Restated outcome: tie each replay server to its owning test run so a killed runner cannot leave a live server, including during an incomplete request. Keep replay behavior and consumers unchanged.

Reproduction on main `08b74a340658f2c30e158fd00459330baebacd74`: job `2229/1792409825690-58547` ran the inline Python runner-kill driver; the complete executable invocation is retained in that job. The driver entered the real `replay_environment`, sent a partial POST, killed its runner, and requested `GET /healthz`. Output: `runner_exit=-9`, server reparented to PID 1, `health_after_runner_death=200`. Owned cleanup confirmed `cleanup_port_closed=true` and `cleanup_server_gone=true`.

Exact implementation paths: `tools/replay-server`, `tests/test_replay_server_lifetime.py`, `CHANGELOG.md`, plus the already-recorded 2111 dependency. Check actual Bash process-substitution ownership before finalizing parent monitoring. Existing context finally, Bash traps and Rust Drop do not handle a killed runner. Red job `2229/1792410174697-85243` demonstrated both Python and actual Bash ownership failures, while normal exit and timeout passed. The shared watcher fixed Python; green attempt `1792410204381-89126` exposed the intermediate Bash process-substitution shell, which requires replacing that shell with the server via exec.

Outside-in red test: kill a child runner inside a real replay context during a partial request, then require its server process to disappear and port to close. Cover normal exit and timeout unwinding in the same focused module. Put prevention in the shared server; change launchers only if their real ownership demands it. Run focused checks through the build job; no product CLI rebuild is needed for this Python helper change.

Landing gate `2229/1792410600734-72626` stopped before publication on the existing binary-driver classifier: the Python-only fixture sets `BIOMCP_BIN` to `sys.executable`. 1,319 other cases passed, ten skipped; main remained unchanged. The reviewer corrected the existing explicit fake-tool exemption rather than adding a false binary marker. Focused job `2229/1792411044979-94669` passed classification, all four lifetime cases and mechanical checks.

## Progress

- 2026-10-19 started
