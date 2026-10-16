# 2115 — Count sdlc programs as changes in the CI classifier

Status: OPEN.

Milestone: 0.9.3

## Outcome

A push that changes a program, gate wrapper or configuration under `sdlc/` runs the full CI jobs. Only record and planning text under `sdlc/` still counts as docs-only.

## Evidence

- `scripts/ci-classify-push.sh:76` treats every `sdlc/*` path as docs-only, so a push touching only `sdlc/` skips the build, lint, Rust, Python-lane, spec and other jobs gated on `docs_only` in `.github/workflows/ci.yml`. The repository-contracts job still runs the Python tests that need no binary.
- `sdlc/` holds programs and configuration, not only records: `sdlc/scripts/{clean,lint,spec,test}`, `sdlc/project/{before,failure,health,success,tasks,provenance.json}`, `sdlc/pm.json`, `sdlc/skills/**` and `sdlc/release-checklist.md`. `tests/test_sdlc_project_lifecycle_contract.py` exercises the lifecycle programs.
- Found by a build-efficiency review on 2026-10-16.

- Starts from: `scripts/ci-classify-push.sh`, `tests/test_ci_classify_push_script.py`.
- Decisions:
  - Docs-only under `sdlc/` becomes an allow-list of text folders: `sdlc/tickets/**`, `sdlc/records/**`, `sdlc/decisions/**`, `sdlc/designs/**`, `sdlc/ideas/**`, `sdlc/issues/**`, `sdlc/planning/**` and `sdlc/plan.md`. Every other `sdlc/` path counts as a change. Tests that read a file in those folders run in the repository-contracts job, which always runs.
  - `sdlc/pm.json` declares no `inertPaths` (pm 79f5f7c3, 2026-10-16). Every `sdlc/` folder has a reader: the changelog-coverage and documentation-audit checks read tickets and records, tests read issues, planning and skills, and the whole-tree scanners (`tools/check-zero-coupling.py`, `tools/check-tracked-text`) read every tracked file. The CI allow-list above can skip the Rust jobs because the repository-contracts job, which runs those readers, always runs; pm's kept-pass reuse has no such second job. Revisit only if 2116 measures a real cost.
- Changes: claims `scripts/ci-classify-push.sh` `tests/test_ci_classify_push_script.py` `CHANGELOG.md`.
- Proof: classifier tests, failing on current main, show a push touching only `sdlc/scripts/test`, `sdlc/project/health` or `sdlc/pm.json` is not docs-only, and a push touching only `sdlc/tickets/` and `sdlc/records/` still is.
- Order: lands before 2094, which reshapes `ci.yml`; touches no file 2094 claims except `CHANGELOG.md`.
