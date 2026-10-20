---
priority: 2
---

# 2311 — Declare the focused Python tool environment

Status: OPEN.

Milestone: 0.9.4

Depends on: 2308

Reviews: revision 6cbe728, accept

Reviews: revision 957e895, accept

## Outcome

One existing declared Python bootstrap supplies pinned Ruff beside pytest. Focused checks use one documented command convention and their own job environment; they never borrow another ticket environment. Keep the established Ruff version as one source of truth and preserve ordinary lint failures. Add no release gate or cleanup framework.

Changes: `.github/actions/lane-setup/action.yml`, `.github/workflows/ci.yml`, `tests/test_upstream_planning_analysis_docs.py`, `pyproject.toml`, `uv.lock`, `Makefile`, `bin/lint`, `AGENTS.md`, `tests/test_lint.py`, `CHANGELOG.md`, `sdlc/records/2311-focused-python-environment.md`.

## Evidence

The 2299 root cause record names incomplete job environments. Bounded follow-up RCA confirms missing executable setup failures in 2307/1792527914259-1241717 and 1792530775353-1315640; 2302/1792531260828-99502 instead used a Python without the Ruff module, followed by a distinct missing-PATH-command retry; 2310/1792531804580-1467366 passed its cases then failed when its borrowed 2308 environment lacked Ruff. The 2300 tracked-text execution failure is a separate command-invocation error and must not be called missing Ruff.

## Proof

Before code, a fresh isolated job environment reproduces the missing declared Ruff capability without falling back to a globally installed tool. The existing bootstrap then supplies pytest and the exact required Ruff in that job environment; both commands run through the documented convention. A deliberately foreign environment cannot silently supply the tool. Correct version and command failures remain visible. Run affected checks and the required routine through pm job, then obtain fresh review.

## Order

PM requires this 0.9.4 process fix before release rehearsal. Develop alongside reliability work and preserve the accepted 2308 security pins. It blocks no accepted product landing. Coordinate any shared lint entry-point changes with 2307; do not rewrite retained experiment scripts or old evidence.
