# 2045 — make the article search deadline test fail instead of hang

Status: OPEN.

Milestone: 0.9.2

## Outcome

A bare `cargo test` run of the article-search deadline test fails at a bounded time with the test's name. It never hangs the runner waiting on a reply that never arrives.

## Evidence

Filed 2026-10-10 from Ian's ruling (decisions/2026-10-09-biomcp-0-9-2-is-a-clean-release.md, item 9): two local runs of the article-search deadline test sat idle using no CPU — one for 2 days 4 hours after its worktree was deleted, one for almost 9 hours — and had to be killed by hand. Ticket 2030's per-test kill budget covers nextest runs (`.config/nextest.toml` slow-timeout with terminate-after), and CI runs nextest, so CI fails such a hang at 240 seconds. A bare `cargo test` run uses no such budget, so the blocking wait inside the test waits forever.

- Starts from: ticket 2030's async fixture holds and the 2038 code lane's kill budget.
- Keeps: the nextest kill budget and the async holds.
- Changes: the test's own wait carries its own bound (a timeout on the held reply at the test level, or the deadline firing ends the wait), so even a bare run fails in bounded time with the test's name.
- Proof: a bare `cargo test` run of the deadline module on the build machine, under the checkout lock, finishes with a failure naming the test inside minutes instead of hanging.
- Defers: nothing.
