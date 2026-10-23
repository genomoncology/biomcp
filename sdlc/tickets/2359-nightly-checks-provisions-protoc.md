# 2359 — Nightly checks provision their own protoc

Status: OPEN.

Milestone: 0.9.4

Priority: 2

Changes: scripts/run-nightly-checks.sh, Makefile

## Outcome

The scheduled nightly-checks job provisions the pinned protoc it needs, the way the lint step bootstraps its pinned cargo-deny, so it runs on any Linux build machine without hand-installed tools. scripts/run-nightly-checks.sh claims the scheduler prepares protoc; the pm scheduler prepares no tools, and the GitHub workflows install protoc with arduino/setup-protoc.

## Evidence

- Scheduled job j5171 on Gator failed with FileNotFoundError: 'protoc' after Jarvis pinned nightly-checks to Gator (m3509). The earlier j4432 failure on the M5 was the Linux-only stress step, now resolved by that placement.

- Proof: a nightly-checks run on Gator from a checkout with no protoc on PATH passes the protobuf generation check, and the script comment matches what provides the tool.
