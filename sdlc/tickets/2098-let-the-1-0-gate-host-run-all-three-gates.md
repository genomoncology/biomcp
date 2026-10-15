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
- Cause 4 (test): `tools/run-offline` refuses macOS by design, so `make test` cannot run as-is on M5; lanes run the underlying gate commands unsandboxed instead.
- Cause 3 (environment, needs an owner decision): `website/prepare` pins Node v22.22.3 while M5 carries only v26; either M5 installs a v22 toolchain or the pin moves — not decidable by a lane.

## Work

Fix causes 1 and 2 as code: the lint bootstrap gains a Darwin-arm64 case; the spec scripts read process data portably or degrade explicitly when `/proc` is absent. Cause 3 goes to the PM with options; it blocks `make test` only through the website prepare step.

## Acceptance

1. On M5, `make lint`, `make test`, and `make spec` each run to their own verdicts (pass or real findings) at the 1.0 line tip, in one pm job, with no step exiting on a missing platform.

## Ruling 2026-10-15: Node pin and the offline sandbox on macOS

The PM rules on the two gaps a lane could not decide.

- Cause 3 (Node): keep the `website/prepare` pin at v22.22.3. The 1.0 line merges `main` continuously, so a pin that moves on 1.0 alone diverges from `main` and from CI on every merge. The gate host installs Node v22.22.3 in the user's own toolchain directory beside its current Node, and `website/prepare` finds it the way it does on Linux. Moving the pin is a `main` ticket for a later release.
- Cause 4 (offline sandbox): `tools/run-offline` keeps its Linux network sandbox. On macOS it runs the command without the sandbox and prints one line to stderr saying the run is not network-isolated, instead of refusing. Linux CI stays the proof that the suite runs offline. A test that reaches the network on macOS while CI stays green is a test defect, filed as its own ticket.
- Changes: adds `tools/run-offline` to this ticket's work beside the lint bootstrap and the two spec scripts.
- Proof: acceptance 1 stands, and the `make test` log on the macOS host carries the not-network-isolated line.

## Coordinator rulings on the lane's questions

- Node@22 precedence machine-wide on M5: accepted, per the PM's Node-pin ruling; node@26 stays available via its keg. Record the mechanism here, not in biodata.
- Gate-host rustfmt/clippy pinned to CI's versions: yes — gate parity includes toolchain parity. The M5 lint findings are presumed skew until the pin lands; GitHub CI (Linux toolchain) on this branch is the arbiter for this landing.

Deferrals: the failing Rust test -> 2106; the 14 macOS spec pages -> 2107.

