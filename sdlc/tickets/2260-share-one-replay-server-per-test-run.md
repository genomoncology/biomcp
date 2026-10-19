---
priority: 2
---

# 2260 — Share one replay server per test run

Status: OPEN.

Milestone: 0.9.4

## Outcome

Changes: `sdlc/tickets/2251-report-publisher-elided-references-as-empty.md`, `sdlc/tickets/2252-make-variant-articles-markdown-match-its-json.md`, `sdlc/tickets/2253-document-full-text-access-groups-and-reason-codes.md`, `sdlc/tickets/2254-fix-article-help-lists-and-condition-names.md`, `sdlc/tickets/2256-bound-only-the-mcp-handshake-wait-by-the-startup-timer.md`, `sdlc/tickets/2257-route-every-test-cache-root-through-the-shared-guard.md`, `sdlc/tickets/2258-admit-the-0-9-4-release-candidate.md`, `sdlc/tickets/2259-check-replay-recordings-against-live-providers-before-release.md`, `tests/golden/replay_env.py`, `tests/conftest.py`, `tests/golden/conftest.py`, `tests/test_variant_article_transcript_aliases.py`, `tests/disable_sources_cli.rs`, `tests/test_replay_server_lifetime.py`, `tools/replay-server`, `CHANGELOG.md`, `sdlc/tickets/2260-share-one-replay-server-per-test-run.md`.

Each test run starts at most one replay server per test process tree, reused by every test that needs it, and that run owns its teardown, including when the run is killed.

## Evidence

On 2026-10-19, 76 orphaned `tools/replay-server` processes from BioMCP job checkouts were found on the build machine, all started before 2229, which made the server exit with its parent. Today several places still start their own server: `scripts/run-specs.sh` (one per spec run), `tests/golden/replay_env.py` used per test module by `tests/golden/conftest.py` and six other test files, `tests/test_variant_article_transcript_aliases.py`, and `ReplayServer::start` in `tests/disable_sources_cli.rs`.

## Keeps

Keep parent-death exit and per-test request-log isolation (tests read only their own slice of the log). Tests that deliberately test the server's own lifetime may start their own. No test changes its assertions.

## Proof

A session-scoped pytest fixture (one per worker under parallel runs) and one shared server per Rust test binary replace the per-module starts. A check counts replay-server processes during a full `make test` and `make spec` and finds at most one per runner process, and none after the run, including after killing the runner mid-run.

## Progress

- 2026-10-19 started

## Coordinator implementation grant

Approved implementation: runner-owned shared process with per-context isolated cache/work roots and request-log baselines, function-scoped pytest views over the worker session owner, and serialized Rust views over one binary owner. Preserve every assertion and parent-death handling. Allow an explicitly enabled loopback-only fixture-root switch under the context lock to preserve projected fixtures; it must not become a recorded provider request or change ordinary server behavior. Wait for the active 2257 writer to stop before common fixture edits; integrate its shared cache policy first. Targeted lifetime and isolation proof now; whole-run counting proof on the reviewed combined candidate.

## Implementation evidence

Developer stopped at a68d58f99f4c8243eaedb3c7c3186e4a43f654f6 with a clean published branch. Shared owner proof includes five expected baseline failures and nine passing lifecycle, root-switch and mixed-mode cases; sixty-eight affected Python checks and four Rust server cases passed, with formatting, Ruff and warnings-denied Clippy. An incompatible prepared binary caused three fixture failures; those three passed against the source-compatible artifact. The sole inherited quality failure was separately fixed by accepted 2257 revision c905fd235 and adopted here. No whole-run process-count or final live claim.

Keep: experiments/804-biomcp-2260-run-owned-replay — retain tested patches, commands, source and binary hashes and raw red/green results.

The opt-in context switch accepts fixture root and recording mode under the owner lock, restores both at exit and keeps explicit golden recording separate from nonrecording callers. Whole combined test/spec process counts and killed-run cleanup remain required before closing the ticket. Carried sibling records are unchanged approved PM ancestry, not claimed implementation.
