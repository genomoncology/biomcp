---
priority: 1
---

# 2186 — Prepare release qualification lanes in clean checkouts

Status: OPEN.

Milestone: 0.9.3

## Outcome

Release qualification prepares the dependencies and temporary directories each canonical command needs in a clean checkout before it enters offline execution. Keep local reuse and the existing gates intact.

- Evidence: full nonpublishing rehearsal 39076922723 at eb39028ea10dd973fa86aaeaea887d41fbafcedc failed its Python lane before any test ran: make test-python entered offline uv run --no-sync pytest with no prepared environment; pytest was absent. Its specification lane also failed the existing Routine gates select the smaller feature set case because mktemp received the canonical routine TMPDIR before that directory existed. Both passed on the reused local job checkout, so existing local qualification missed clean-checkout preparation.
- Starts from: the canonical command preparation and release qualification workflow callers. Inspect the clean-checkout preconditions before editing.
- Changes: claims `.github/workflows/ci.yml` `Makefile` `spec/surface/build-profile.md` `tests/test_ci_workflow_contract.py` `CHANGELOG.md`; extend only to the existing focused preparation/command contract if inspection shows it owns the behavior, and report that before editing.
- Proof: retain the two failed rehearsal job logs as actual RED evidence. Reproduce each missing-preparation boundary with the existing outside-in command checks on current main through a build job. Add or strengthen only the smallest existing check that catches these clean-checkout failures. Run affected checks, obtain fresh review, land through the configured gate, check actual main, then run the full manual nonpublishing rehearsal on the repaired candidate.
- Keeps: offline isolation, dependency pinning, prepared artifact reuse, routine feature selection, warning policy, all product assertions and release-only GitHub triggers.
- Defers: unrelated workflow refactoring, broad measurement recollection, changing timeouts or ignoring failed gates. Existing measurements remain applicable unless the affected command changes their actual workload; report any such impact explicitly.

## Reproduction

- Candidate: eb39028ea10dd973fa86aaeaea887d41fbafcedc.
- Python job 117526743616: make test-python; error Failed to spawn pytest, No such file or directory; exit 2. No Python tests ran.
- Specification job 117526743547: make spec; build-profile.md line33 failed because routine-test-tmp did not exist for mktemp. Sixty Markdown cases and six golden cases passed, one Markdown case failed.
- Root retains the complete job logs in temporary scratch. The rehearsal run URL is https://github.com/genomoncology/biomcp/actions/runs/39076922723 .
