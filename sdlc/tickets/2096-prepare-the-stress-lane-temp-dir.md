# 2096 — Prepare the stress lane temp dir

Status: OPEN.

Milestone: 0.9.3

## Outcome

The stress lane on main runs its pytest rounds without dying on a missing temp dir.

## Evidence

Filed 2026-10-15 from main's red run after the CI restructure landed: the stress lane (now main-only, so its branch never ran it) failed with pytest INTERNALERROR, FileNotFoundError for `.cache/routine-test-tmp/pytest` — the stress target uses the prepared basetemp but never depended on the target that creates it; every other test lane does.

- Starts from: the `stress` target in the Makefile; `prepare-routine-test-tmp`.
- Changes: `stress: prepare-routine-test-tmp` — one prerequisite line.
- Proof: branch CI green with the stress lane exercising the prepared dir.
- Defers: nothing.
