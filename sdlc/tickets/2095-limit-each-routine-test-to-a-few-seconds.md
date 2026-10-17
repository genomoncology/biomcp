---
priority: 3
---

# 2095 — Warn when routine tests exceed the slow-test threshold

Status: COMPLETE.

Milestone: 0.9.3
Depends on: 2094
Depends on: 2159

Depends on: 2156

Reviews: revision 5b7f6dd, accept

Reviews: revision 3c79683, accept

Reviews: revision 6fc832b, accept

Landed: 6fc832b (fast-forward: Accepted source landed inside carrier 61a8b075ef075e5650dbb85f8572ee8a341803ad with shared gate1792256340018-55751; individual review retained and actual-main full check follows the batch.)

## Outcome

A routine Rust or Python test exceeding thirty seconds emits a warning identifying the test and machine load; elapsed time alone never fails or terminates it.

- Starts from: `.config/nextest.toml`, the shared pytest runner and the routine/long suite selections.
- Additional reviewed claims: `tools/report-slow-tests.py`, `tests/test_slow_test_reporting.py`, `tests/test_python_contract_parallelism.py`, `tests/test_upstream_planning_analysis_docs.py`, `tests/test_routine_cargo_feature_contract.py`.
- Keeps: assertion failures fail; exhaustive tests have an explicit long-suite command and may take the time they need; meaningful tests are not deleted because they are slow.
- Changes: claims `.config/nextest.toml` `tests/conftest.py` `Makefile` `pyproject.toml` `AGENTS.md` `CHANGELOG.md`. Add nonterminating Rust slow-test reporting and Python duration reporting with load. Name affected runner-contract files in this claim before changing them. Remove the proposed ten-second termination policy and its timeout exceptions. Keep the threshold in its runner configuration rather than duplicating it in instructions.
- Proof: through the actual routine runner commands, planted passing Rust and Python cases exceeding the warning threshold emit test identity and load and still pass; a planted assertion failure fails. Verify long-suite selection separately. Run these proofs through the build job without unrelated exhaustive suites.
- Defers: exhaustive execution to the release candidate and changes to covered behavior.

PM decision: 2026-10-17. Ian confirmed warning-only routine timing and a separate long suite. The dependency on 2094 serializes edits to shared runner files; it does not reinstate a hosted CI gate.

PM decision: 2026-10-17. This ticket waits on 2094 only to serialize edits to shared runner files.

Coordinator dependency correction: the warning implementation adds a reporting tool and its contract module. The existing exact package-count check would reject those legitimate files, so the already accepted derived-membership fix 2159 must land before this change. Its source check remains enforced; no temporary counter update is needed.
