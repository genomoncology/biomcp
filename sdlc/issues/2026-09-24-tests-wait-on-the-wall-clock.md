# Tests wait on the wall clock

Filed 2026-09-24 from the review of tickets 1239, 1247, and 1248. One cause sits under the gate-host flakes: tests wait for events with fixed deadlines, polling loops, and short samples, and run in parallel on the 4-core gate host. The raw ctgov SIGABRT is a different problem: it is deterministic on one host and not load-driven.

## Evidence

- 1239 raised the GenCC lease child's wait from 5 s to 120 s of polling (`src/sources/gencc/tests.rs:849`, `src/entities/gene/gencc/tests.rs:915`). A slower host can still exceed it, and a parent panic leaves the child alive for up to 120 s.
- 1247 waits against a 60 s deadline. A leaked temp now fails only after 60 s with a generic message (`src/sources/gencc.rs:863`), because the promised follow-up check was cut to stay under the file-size limit.
- 1248 blames "suite processes racing the scan" without evidence. The failing check is one 200 ms heartbeat sample (`tests/test_disease_survival_fixture_lifecycle.py:45-48`, `:193`). It cannot tell a slow decoy from a decoy the fixture wrongly killed. The second would be a real bug.
- The reviewer counted 11 fixed-deadline lines in 7 Rust test files and 77 `sleep` or `timeout=` calls in Python tests. Those are only the matches for two patterns.

## Fix once

- Replace waits with events. For the GenCC lease child: the child writes "entered" to stdout and blocks reading stdin. The parent reads that line, then closes stdin to release it. If the parent dies, the child reads end-of-input and exits. Keep the new "child still running" check.
- For 1248: check the decoy through `/proc/<pid>/stat` for alive and not a zombie, and wait on the stale process being gone. Rewrite ticket 1248 on this basis.
- Add shared wait helpers built on pipes, notifications, or process handles.
- Scale every remaining watchdog timeout by one factor, `BIOMCP_TEST_TIMEOUT_SCALE`.
- Add a lint to `make lint` that flags new `Instant::now() + Duration::from_secs`, `time.sleep`, and single heartbeat samples in tests unless the line carries a `watchdog:` reason.
- Add a stress lane that runs the known-flaky tests pinned to one CPU, so a fix is proven by reproduction, not by three green runs.

## Resolved

Ticket 1252 (with 1248 rewritten on the new basis). The lease waits
are pipe handshakes; the settle failure names its leaked paths; the
reap check reads /proc stat; helpers carry BIOMCP_TEST_TIMEOUT_SCALE;
a wait ratchet pins the remaining sleeps per file; make stress runs
the flaky set on a two-CPU set with forced parallelism. The one-CPU
pinning variant deadlocks the handshake child and is filed open with
probe evidence. See
`sdlc/records/1252-replace-timed-waits-with-signals-and-add-a-stress-lane.md`.
