# 2045 — make the article search deadline test fail instead of hang

Status: OPEN.

Milestone: 0.9.2

## 2047 finding: the observed hang's real location

Ticket 2047 reopened the location question: every deadline test already
runs inside a 60-second tokio watchdog, so the 180-second bound on the
held reply cannot change any existing test's result, and the red test is
built to hang rather than reproduce the observed one. The investigation
ran on the build machine under the checkout lock (branch
`probe/2047c-hang`, main with the 180-second bound reverted).

- A live specimen of the observed hang was still on the build host: a
  nextest archive run from 2026-10-07 whose single-test child
  `--exact entities::article::search::tests::deadline::single_backend_deadline_keeps_fetched_rows_and_names_the_source
  --nocapture` had sat idle for 2 days 12 hours with zero CPU after its
  `/tmp/nextest-archive-*` tree was deleted. Its parent nextest never
  killed it: the per-test kill budget reached the archived run only with
  `e4f10fad3` on 2026-10-09, after this run started. Both orphans were
  killed at the end of this investigation.
- The specimen's state, captured from `/proc` and gdb: six threads — the
  main thread, the test thread, and four `tokio-rt-worker` threads — all
  parked in `futex_do_wait`, none runnable, no timer due. The test thread
  was still inside its `#[tokio::test]` body: the future never completed
  and no watchdog ever fired. With every worker parked, a tokio timer
  cannot fire at any bound, which is why the 60-second watchdog and the
  180-second hold bound are both inert against this hang: it sat outside
  every timer's reach, not just outside the timed body.
- The wait that blocked: the pre-2026-10-08 held reply
  (`std::sync::Mutex<std::sync::mpsc::Receiver<()>>` with a blocking
  `lock().recv()`) ran inside `tokio::spawn`, parking a runtime worker
  thread per held connection. The client's 30-second timeout made the
  held leg error and retry (three retries), and each retry opened a new
  connection whose server-side handler parked another worker on the
  unbounded std receive. Four attempts parked all four workers of the
  tests' four-worker runtime, freezing every timer — including the
  client timeout that would have broken the cycle. The run then waited
  forever with no CPU.
- The fix location: ticket 2030's `925f8a0e1` (2026-10-08, "Wait on
  async primitives in the article fixture holds") replaced the std
  mutex and receiver with a tokio mutex and mpsc receiver, so a hold
  parks an async task instead of a worker and the runtime's timers keep
  firing. That commit's message names the exact mechanism ("four held
  connections froze every timer on the deadline tests' four-worker
  runtime"). The observed hang class is closed on main by that commit,
  one day before this ticket was filed.
- Reproduction on current main: twenty-five bare runs of the deadline
  module (`--no-default-features --locked --lib`, cold target dir, fix
  reverted on the probe branch) and one bare full-lib run all finished
  green (11 tests in 12–16 seconds; the full run 418 seconds with two
  unrelated `sources::tests` network flakes). No hang reproduced with
  the async holds in place.
- The 180-second bound stays. It is inert for every legitimate test
  (every search deadline and watchdog sits at 60–120 seconds, far under
  it), it changed no existing test's result, and it is the only bound a
  bare run has for the residual class this ticket's red test models: a
  held reply whose caller outlives its test with no sender left. The
  observed hang needed none of it — it needed workers that never park,
  which ticket 2030 delivered.

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
