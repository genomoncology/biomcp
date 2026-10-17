---
priority: 1
---

# 2186 — Prepare release qualification lanes in clean checkouts

Status: OPEN.

Milestone: 0.9.3

Reviews: revision 6477ba6, accept

## Outcome

Release qualification prepares the dependencies and temporary directories each canonical command needs in a clean checkout before it enters offline execution. Keep local reuse and the existing gates intact.

- Evidence: full nonpublishing rehearsal 39076922723 at eb39028ea10dd973fa86aaeaea887d41fbafcedc failed its Python lane before any test ran: make test-python entered offline uv run --no-sync pytest with no prepared environment; pytest was absent. Its specification lane also failed the existing Routine gates select the smaller feature set case because mktemp received the canonical routine TMPDIR before that directory existed. Both passed on the reused local job checkout, so existing local qualification missed clean-checkout preparation.
- Starts from: the canonical command preparation and release qualification workflow callers. Inspect the clean-checkout preconditions before editing.
- Changes: claims `.github/workflows/ci.yml` `Makefile` `spec/surface/build-profile.md` `tests/test_ci_workflow_contract.py` `tests/test_routine_cargo_feature_contract.py` `CHANGELOG.md`; extend only to the existing focused preparation/command contract if inspection shows it owns the behavior, and report that before editing.
- Proof: retain the two failed rehearsal job logs as actual RED evidence. Reproduce each missing-preparation boundary with the existing outside-in command checks on current main through a build job. Add or strengthen only the smallest existing check that catches these clean-checkout failures. Run affected checks, obtain fresh review, land through the configured gate, check actual main, then run the full manual nonpublishing rehearsal on the repaired candidate.
- Keeps: offline isolation, dependency pinning, prepared artifact reuse, routine feature selection, warning policy, all product assertions and release-only GitHub triggers.
- Defers: unrelated workflow refactoring, broad measurement recollection, changing timeouts or ignoring failed gates. Existing measurements remain applicable unless the affected command changes their actual workload; report any such impact explicitly.

## Reproduction

- Candidate: eb39028ea10dd973fa86aaeaea887d41fbafcedc.
- Python job 117526743616: make test-python; error Failed to spawn pytest, No such file or directory; exit 2. No Python tests ran.
- Specification job 117526743547: make spec; build-profile.md line33 failed because routine-test-tmp did not exist for mktemp. Sixty Markdown cases and six golden cases passed, one Markdown case failed.
- Root retains the complete job logs in temporary scratch. The rehearsal run URL is https://github.com/genomoncology/biomcp/actions/runs/39076922723 .

## Progress

- 2026-10-17 started

## Repair and checks

Restated outcome: make each clean release lane prepare its own command prerequisites before offline execution, without rebuilding the prepared Rust artifacts. The canonical Python target owns dependency preparation; the existing feature probe owns its private temporary directory.

The developer reported the additional existing routine-feature contract module before editing, and the coordinator authorized it. Five files changed: Makefile, the build-profile specification, the two existing contract modules and the changelog. The workflow itself did not change.

- Focused RED, job 1792278625319-71044 at published base 5b1cf286a plus tests-only patch: `make sync-python-dev`, then `uv run --no-sync pytest tests/test_ci_workflow_contract.py::test_python_gate_prepares_a_clean_environment_before_offline_execution tests/test_routine_cargo_feature_contract.py::test_routine_feature_probe_runs_without_a_prepared_checkout -q -n 0`. Two failed: the clean Python command had no prepared environment, and the feature probe selected the absent default temporary path.
- Focused GREEN, job 1792278656130-72055 at the same base plus final patch: `make sync-python-dev`; `uv run --no-sync pytest tests/test_ci_workflow_contract.py tests/test_routine_cargo_feature_contract.py tests/test_python_contract_parallelism.py -q -n 2`; `ruff check tests/test_ci_workflow_contract.py tests/test_routine_cargo_feature_contract.py`; `mustmatch lint spec/surface/build-profile.md --min-like-len 10`; `git diff --check`. Forty-six passed, one skipped; lint and whitespace checks passed. Incremental compilation was unset and worker limits were two.
- Tested patch SHA256 995d9b9e687a350ad6d3f8b8ef12414df58a26d48b38dbdc01dbf2ca900eb939 matches the committed diff from 5b1cf286a to 6c7b4a7bef75f48e1007094286ba701fd4e69765; coordinator checked that equality and the job output.
- The local operating system does not reproduce GNU mktemp behavior directly. The failed Linux rehearsal remains direct RED; the next required full rehearsal must supply Linux GREEN.

Root cause: independent release commands used prerequisites that a reused local checkout already held. The fix makes the Python target sync its environment before entering offline execution and gives the dry-run feature probe an existing private directory with exit cleanup. Offline isolation and all product assertions remain intact.

Measurement applicability: previously recorded observations retain their measured revisions and exact commands. The revised test command adds a dependency-sync check and the revised specification probe adds directory preparation, so the earlier test/spec timings do not measure these revised workloads. Product source, compiler inputs, runtime scenarios and coverage instrumentation did not change; reuse their existing evidence. The required final main check and full rehearsal qualify the repaired commands; no new performance claim is made.
