# Code Review Log

## Critique

- Reviewed `.march/design-final.md`, `.march/design-draft.md`, `.march/code-log.md`, `.march/ticket.md`, the full `git diff main..HEAD`, and the single ticket commit `f74a61d0`.
- Design completeness and proof traceability were otherwise intact: the branch updated the canonical prompt in `skills/SKILL.md`, tightened the direct prompt contract in `tests/test_public_skill_docs_contract.py`, and extended the rendered-surface executable spec in `spec/surface/discover.md`.
- **Blocking defect found:** `docs/user-guide/discover.md` still listed `biomcp discover "drug classes that interact with warfarin"` as a normal example even though the new shipped prompt guidance treats that exact query as a discover anti-pattern and redirects the user toward article search instead. That left adjacent help/docs inconsistent for the same changed contract.

## Fix Plan

1. Remove the stale relational-query example from `docs/user-guide/discover.md`.
2. Replace it with explicit single-entity examples that match the new prompt framing (`BRCA1`, `dabigatran`).
3. Add a discover-guide contract assertion so the general discover docs cannot drift back to advertising the warfarin relational query as a normal discover example.

## Repair

- Updated `docs/user-guide/discover.md` to show `biomcp discover BRCA1` and `biomcp discover dabigatran` in the examples block and removed the stale warfarin relational-query example.
- Updated `tests/test_documentation_consistency_audit_contract.py` to require those positive examples and to reject the stale `biomcp discover "drug classes that interact with warfarin"` example.
- Re-checked the touched area for collateral damage after the fix; no dead branches, stale text, or other adjacent issues were introduced.
- Re-ran the changed-surface and repo gates:
  - `cargo test --lib`
  - `cargo clippy --lib --tests -- -D warnings`
  - `uv run pytest tests/test_public_skill_docs_contract.py tests/test_documentation_consistency_audit_contract.py -q`
  - `PATH="$(pwd)/target/release:$PATH" BIOMCP_BIN="$(pwd)/target/release/biomcp" uv run --extra dev pytest spec/surface/discover.md --mustmatch-lang bash --mustmatch-timeout 180 -q`
  - `make test-contracts`
  - `make spec-pr`

## Residual Concerns

- None. No out-of-scope follow-up issue was needed from this review.

## Defect Register

| # | Category | Lintable | Description |
|---|----------|----------|-------------|
| 1 | stale-doc | no | `docs/user-guide/discover.md` still advertised `biomcp discover "drug classes that interact with warfarin"` as a normal example after `skills/SKILL.md` reclassified that query as a discover anti-pattern. |
