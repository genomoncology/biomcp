---
flow: build
priority: 2
deps: []
---

# 2098: Let the 1.0 gate host run all three gates

Status: OPEN.

Milestone: 1.0

## Evidence

- Filed 2026-10-15 from lanes 2004 and 2006: every 1.0 lane's acceptance names `make lint`, `make test`, and `make spec` on the gate host, and all three fail on M5 for pre-existing reasons proven at the base tip e2dff0a46, before any lane's diff.
- Cause 1 (lint): `tools/bootstrap-lint-tools` ships no Darwin-arm64 archive and exits before any check runs, so `make lint` cannot start.
- Cause 2 (spec): `scripts/run-specs.sh:7` and `scripts/fixture-supervisor.sh:9` read `/proc` unconditionally, so `make spec`, `spec-static`, and the parallel-isolation pytest lane are Linux-only and cannot run on a macOS gate host.
- Cause 3 (environment, needs an owner decision): `website/prepare` pins Node v22.22.3 while M5 carries only v26; either M5 installs a v22 toolchain or the pin moves — not decidable by a lane.

## Work

Fix causes 1 and 2 as code: the lint bootstrap gains a Darwin-arm64 case; the spec scripts read process data portably or degrade explicitly when `/proc` is absent. Cause 3 goes to the PM with options; it blocks `make test` only through the website prepare step.

## Acceptance

1. On M5, `make lint`, `make test`, and `make spec` each run to their own verdicts (pass or real findings) at the 1.0 line tip, in one pm job, with no step exiting on a missing platform.
