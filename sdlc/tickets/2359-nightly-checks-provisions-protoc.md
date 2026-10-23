# 2359 — Nightly checks provision their own protoc

Status: OPEN.

Milestone: 0.9.4

Priority: 2

- Changes: `CHANGELOG.md`, `scripts/run-nightly-checks.sh`, `tools/bootstrap-lint-tools`, `tests/test_release_check_counterparts.py`, `tests/test_advisory_bootstrap.py`, `src/entities/article/graph/tests.rs`, `src/entities/article/graph/citation_evidence.rs`, `tools/test-wait-exceptions.json`. The candidate deadline-test repair replaces the elapsed-time confirmation setup with a direct expired-deadline boundary test and removes its unused delayed JATS reply and its obsolete fixed-wait exception. `Cargo.lock` carries the accepted 2361 Tokio dependency repair authorized by PM m4030 for this batch candidate. `sdlc/tickets/2361-clear-tokio-asyncfd-advisory.md`, `sdlc/tickets/2371-serialize-installation-mutations.md`, `sdlc/tickets/2372-serialize-release-publication.md` carry coordinator-authorized intake of the reviewed open records for 2361, 2371 and 2372; this intake adds no implementation for 2371 or 2372.

## Outcome

The scheduled nightly-checks job provisions the pinned protoc it needs, the way the lint step bootstraps its pinned cargo-deny, so it runs on any Linux build machine without hand-installed tools. scripts/run-nightly-checks.sh claims the scheduler prepares protoc; the pm scheduler prepares no tools, and the GitHub workflows install protoc with arduino/setup-protoc.

## Evidence

- Scheduled job j5171 on Gator failed with FileNotFoundError: 'protoc' after Jarvis pinned nightly-checks to Gator (m3509). The earlier j4432 failure on the M5 was the Linux-only stress step, now resolved by that placement.

- Proof: a nightly-checks run on Gator from a checkout with no protoc on PATH passes the protobuf generation check, and the script comment matches what provides the tool.
