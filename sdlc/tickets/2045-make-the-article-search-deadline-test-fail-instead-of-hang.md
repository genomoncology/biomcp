# 2045 — make the article search deadline test fail instead of hang

Status: OPEN.

Milestone: 0.9.2


## Outcome

A bare `cargo test` run of the article-search deadline test fails at a bounded time with the test's name. It never hangs the runner waiting on a reply that never arrives.

## Fix lane

Landed on `tickets/2044-fix` (commit `Bound the held fixture reply at its own watchdog`), filed 2026-10-10.

The wait that had no bound of its own was the held fixture reply itself: `TestHttpReply::Hold` parked on the async gate with no timer, so a hold that outlived its test waited forever; nextest's slow-timeout covered only nextest runs, and a bare `cargo test` run had no budget at all. The hold now carries its own scaled bound, `crate::test_support::watchdog(180)`, above every legitimate test bound (the 60-second search deadline and the 60-120-second watchdogs), so it can only trip when the test has already failed its own bounds. At the bound the hold self-releases, closes the connection, and the caller waiting on the reply fails with its test's name instead of parking.

The shipped test `held_fixture_replies_self_release_at_their_own_bound` (in the deadline tests module) pins the bound deterministically: under `start_paused` time it holds a reply whose sender is never dropped, and the caller must see the closed connection as an error rather than a reply or a wait. On main this test hangs forever — paused time cannot advance because no timer exists — which is exactly the hang class this ticket names; on the fix it fails the hold's timer immediately and passes.

Proof, run on the build machine under the checkout lock through `yr`:

- Main plus the planted test (branch `probe/2044-red`, no fix): the bare test binary run `timeout 240 .../biomcp_cli entities::article::search::tests::deadline::held_fixture_replies_self_release_at_their_own_bound --exact` never finished — the harness printed "has been running for over 60 seconds" and the outer timeout killed it at 241 seconds with exit 124. On main the hold has no timer, so paused time cannot advance and the run waits on the reply forever: the hang this ticket names, reproduced deterministically.
- A bare run of the deadline module on main itself (no planted test) passed in 17 seconds (`cargo test --no-default-features --locked --lib entities::article::search::tests::deadline`, 10 tests green). The deadlock Ian's ruling names is intermittent, so the planted hang above is the deterministic evidence.
- On the fix branch the same planted test fails the hold's timer at once under paused time and passes, and the deadline module's whole bare run stays bounded — verified: the planted test passes offline in the fresh lib-test build, and the bare run `cargo test --no-default-features --locked --lib entities::article::search::tests::deadline` finished with 11 tests green (the ten plus the bound test) in 15.16 seconds of test time on the build machine under the checkout lock. `make lint` passes at the fix head and branch CI run 38025131267 is green on every job.

## Evidence

Filed 2026-10-10 from Ian's ruling (decisions/2026-10-09-biomcp-0-9-2-is-a-clean-release.md, item 9): two local runs of the article-search deadline test sat idle using no CPU — one for 2 days 4 hours after its worktree was deleted, one for almost 9 hours — and had to be killed by hand. Ticket 2030's per-test kill budget covers nextest runs (`.config/nextest.toml` slow-timeout with terminate-after), and CI runs nextest, so CI fails such a hang at 240 seconds. A bare `cargo test` run uses no such budget, so the blocking wait inside the test waits forever.

- Starts from: ticket 2030's async fixture holds and the 2038 code lane's kill budget.
- Keeps: the nextest kill budget and the async holds.
- Changes: the test's own wait carries its own bound (a timeout on the held reply at the test level, or the deadline firing ends the wait), so even a bare run fails in bounded time with the test's name.
- Proof: a bare `cargo test` run of the deadline module on the build machine, under the checkout lock, finishes with a failure naming the test inside minutes instead of hanging.
- Defers: nothing.

## Landed-head review (2026-10-10)

- Code review (2045 item at head 21630b5eb): ACCEPT 2026-10-10,
  recorded through pm and normalized here. The held-reply watchdog
  bound and its paused-time pin verified; the bare-run proof (241 s
  kill on main, 15.16 s green on the fix) recorded in this ticket.
