---
flow: build
priority: 4
deps: []
---

# 1227: Retire the stale release facts and the vacuous assertions

## Goal

The architecture document stops describing a workflow that no longer exists, the documentation tests stop containing assertions that cannot fail, and the recorded container base matches the shipped Dockerfile.

## Current Facts

- `architecture/technical/overview.md:67-69` still describes the retired two-step stage/promote workflow.
- `architecture/technical/overview.md:280` says the `release/` Python package holds the staged candidate tooling, and `tests/test_docs_changelog_refresh.py:829` pins that sentence.
- `architecture/technical/overview.md:268` was flagged as stale in an earlier draft, but `Cargo.toml:3` is `0.9.1-dev.1` and `pyproject.toml:7` is `0.9.1.dev1`, so the sentence may be current. Inspect it and name a concrete defect before touching it.
- `tests/test_docs_changelog_refresh.py:824-825` contains `... or True` and `assert True`, which cannot fail.
- `release/container.py:117` records the retired `debian:bookworm-slim` digest while `Dockerfile:1` uses trixie (`debian:trixie-slim@sha256:a99cfc51...`); no test asserts the constant.
- `tests/test_upstream_planning_analysis_docs.py:674-682` and `tests/test_docs_changelog_refresh.py:820-829` pin sentences in the overview, so edits must update those pins deliberately.

## Design

- Replace the two-step prose at `overview.md:67-69` with the single-workflow description and add a guard that fails if the retired wording returns.
- Remove `... or True` and `assert True` while keeping the assertions that can fail, and make each surviving assertion fail against a deliberately wrong sentence.
- Derive `release/container.py`'s base from `Dockerfile:1`'s runtime image, or delete the field if nothing consumes it; update the `overview.md:280` sentence and its pin at `tests/test_docs_changelog_refresh.py:829` with the recorded decision that the retired `release/` package stays on disk but is not the release path.
- Leave `overview.md:268` alone unless a concrete defect is named in the ticket or record.

## Acceptance

1. `overview.md` names the single workflow, no longer describes stage/promote, and a guard fails if the retired wording returns.
2. The vacuous assertions are gone and the surviving documentation assertions still fail on a wrong sentence.
3. `release/container.py`'s recorded base matches `Dockerfile:1`, or the field is gone, and the sentence pin at `tests/test_docs_changelog_refresh.py:829` is updated deliberately.
4. `make lint` and `make test` pass on yellow at the pushed SHA, and CI `canonical-gates` is green on main.

## Out of scope

- Deleting the retired `release/` package.
- The release workflow behavior (tickets 1222 and 1226).

## Complexity

- Level 2 (contract 1, state and timing 0, reach 1, proof 1, cost of error 1 = 4)
- Reasons: explicit documentation pins and one code constant; a wrong sentence is caught by a mutation-checked assertion and corrected locally.

## Review

- Design review: pending
- Code review: pending
