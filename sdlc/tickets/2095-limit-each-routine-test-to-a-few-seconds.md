---
priority: 3
---

# 2095 — Warn when routine tests exceed the slow-test threshold

Status: OPEN.

Milestone: 0.9.3
Depends on: 2094

Depends on: 2156

## Outcome

A routine Rust or Python test exceeding thirty seconds emits a warning identifying the test and machine load; elapsed time alone never fails or terminates it.

- Starts from: `.config/nextest.toml`, the shared pytest runner and the routine/long suite selections.
- Keeps: assertion failures fail; exhaustive tests have an explicit long-suite command and may take the time they need; meaningful tests are not deleted because they are slow.
- Changes: claims `.config/nextest.toml` `tests/conftest.py` `Makefile` `pyproject.toml` `AGENTS.md` `CHANGELOG.md`. Add nonterminating Rust slow-test reporting and Python duration reporting with load. Name affected runner-contract files in this claim before changing them. Remove the proposed ten-second termination policy and its timeout exceptions. Keep the threshold in its runner configuration rather than duplicating it in instructions.
- Proof: through the actual routine runner commands, planted passing Rust and Python cases exceeding the warning threshold emit test identity and load and still pass; a planted assertion failure fails. Verify long-suite selection separately. Run these proofs through the build job without unrelated exhaustive suites.
- Defers: exhaustive execution to the release candidate and changes to covered behavior.

PM decision: 2026-10-17. Ian confirmed warning-only routine timing and a separate long suite. The dependency on 2094 serializes edits to shared runner files; it does not reinstate a hosted CI gate.

PM decision: 2026-10-17. This ticket waits on 2094 only to serialize edits to shared runner files.
