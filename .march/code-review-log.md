# Code Review Log

## Critique

I reviewed the branch against `.march/design-final.md`, the full `main..HEAD` diff, and the ticket/spec/docs proof surface.

### Design completeness audit

- The helper grammar, DDInter local-runtime source lifecycle, health row, help/list/docs/source-map updates, and shared helper/section interaction backend were all present in the diff.
- The proof-matrix test surfaces were present across the Rust CLI/render tests, the executable drug spec, and the Python docs/source contract tests.
- Three defects remained in the implementation:
  1. the shared DDInter report dropped the legacy DrugBank/MyChem interaction descriptions instead of carrying them forward as the design required;
  2. applying the shared interaction report overwrote `Drug.pharm_classes`, which regressed the anchor-drug pharmacology field into an interaction-rollup field; and
  3. the DDInter client evicted and rebuilt its in-process index on every `ready()` call, so the "load once per process" design goal was not actually met.

### Test-design traceability

- Helper grammar/help/list/docs coverage: `src/cli/drug/tests.rs`, `src/cli/tests/next_commands_validity.rs`, `src/cli/tests/next_commands_json_property/variant_drug.rs`, `tests/test_public_skill_docs_contract.py`, `tests/test_upstream_planning_analysis_docs.py`, and `spec/entity/drug.md`.
- DDInter source lifecycle and docs coverage: `src/cli/system/tests.rs`, the health tests in `src/cli/health.rs`, `tests/test_source_pages_docs_contract.py`, `tests/test_source_licensing_docs_contract.py`, `tests/test_public_search_all_docs_contract.py`, and `tests/test_upstream_planning_analysis_docs.py`.
- Structured render / empty-state / section-parity coverage: `src/render/markdown/drug/tests.rs`, the interaction tests in `src/entities/drug/interactions.rs`, and the executable spec in `spec/entity/drug.md`.
- I added regression coverage for the defects found during review so the repaired behavior is now directly asserted.

## Fix Plan

1. Preserve the anchor drug's own `pharm_classes` and derive interaction class summaries from interaction rows at render time instead.
2. Merge legacy DrugBank/MyChem interaction descriptions into the DDInter-backed rows when partner names normalize to the same drug.
3. Only evict the DDInter index cache when a sync actually refreshed files, and keep interaction provenance off empty default drug cards while still attributing DrugBank when description text is present.

## Repairs

- Carried legacy DrugBank/MyChem interaction descriptions into the shared DDInter report.
- Stopped `apply_interaction_report()` from mutating `Drug.pharm_classes`, added `interaction_class_summaries()` as the shared rollup helper, and updated the drug markdown template to render class summaries from interaction rows.
- Tightened interaction provenance so default drug cards do not advertise an interactions section when none is present, while DDInter helper/source attribution now adds DrugBank when description text contributes.
- Changed DDInter cache eviction to happen only after an actual refresh, which restores the intended one-load-per-process behavior for warm bundles.
- Added regression tests in `src/entities/drug/interactions.rs`, `src/render/markdown/drug/tests.rs`, and `src/render/provenance.rs`.

## Residual Concerns

- None. No additional out-of-scope issues were filed from this review pass.

## Defect Register

| # | Category | Lintable | Description |
|---|----------|----------|-------------|
| 1 | data-completeness | no | The shared DDInter report dropped legacy DrugBank/MyChem interaction descriptions, leaving the new description column empty even when prior data existed. |
| 2 | collateral-damage | no | `apply_interaction_report()` overwrote the anchor drug's `pharm_classes`, which regressed the existing drug contract into interaction rollups. |
| 3 | performance | no | `DdinterClient::ready()` evicted and rebuilt the DDInter index on every call, defeating the intended in-process cache. |
