---
priority: 2
---

# 2313 — Delete the old pipeline scripts and their tests

Status: COMPLETE.

Milestone: 0.9.4

Reviews: revision 74935ef, accept

Reviews: revision 64c052d, accept

## Outcome

`sdlc/project/` and `sdlc/scripts/` are removed together with the contract tests that only keep them alive. This replaces 2306's plan to move those scripts out of sdlc/.

- Proof: Neither folder exists; the routine suite passes; grep finds no reference to them outside archived records.

## Evidence

They are leftovers of the old automated pipeline, kept alive by contract tests, and they cause nine pm lint findings. From Ian's process assessment of 2026-10-20.

- Changes: `sdlc/project/success`, `sdlc/project/provenance.json`, `sdlc/project/tasks`, `sdlc/project/failure`, `sdlc/project/before`, `sdlc/project/health`, `sdlc/scripts/README.md`, `sdlc/scripts/spec`, `sdlc/scripts/clean`, `sdlc/scripts/test`, `sdlc/scripts/lint`, `tests/test_sdlc_project_lifecycle_contract.py`, `tests/test_sdlc_clean_contract.py`, `scripts/ci-classify-push.sh`, `tests/test_ci_classify_push_script.py`, `CHANGELOG.md`.
