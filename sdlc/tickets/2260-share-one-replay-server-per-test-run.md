---
priority: 2
---

# 2260 — Share one replay server per test run

Status: OPEN.

Milestone: 0.9.4

## Outcome

Each test run starts at most one replay server per test process tree, reused by every test that needs it, and that run owns its teardown, including when the run is killed.

## Evidence

On 2026-10-19, 76 orphaned `tools/replay-server` processes from BioMCP job checkouts were found on the build machine, all started before 2229, which made the server exit with its parent. Today several places still start their own server: `scripts/run-specs.sh` (one per spec run), `tests/golden/replay_env.py` used per test module by `tests/golden/conftest.py` and six other test files, `tests/test_variant_article_transcript_aliases.py`, and `ReplayServer::start` in `tests/disable_sources_cli.rs`.

## Keeps

Keep parent-death exit and per-test request-log isolation (tests read only their own slice of the log). Tests that deliberately test the server's own lifetime may start their own. No test changes its assertions.

## Proof

A session-scoped pytest fixture (one per worker under parallel runs) and one shared server per Rust test binary replace the per-module starts. A check counts replay-server processes during a full `make test` and `make spec` and finds at most one per runner process, and none after the run, including after killing the runner mid-run.

## Progress

- 2026-10-19 started
