# 2154 — Restore the empty LitVar autocomplete fixture

Status: OPEN.

Milestone: 0.9.3

## Outcome

An annotation search whose entity and variant autocomplete replies are both empty reports the existing no-entity miss without a fixture HTTP error, and the shared handler passes test-target lint.

- Starts from: `src/entities/article/variant_search.rs`; the no-entity annotation regression lost its variant autocomplete route in a lint repair.
- Keeps: both autocomplete endpoints return an empty JSON array in this fixture; product request behavior and other fixture replies stay unchanged.
- Changes: `src/entities/article/variant_search.rs` `CHANGELOG.md`.
- Proof: reproduce `no_entity_annotation_run_names_the_miss_instead_of_complete_empty` on current main through the build job, preserve its failing output, restore both route predicates with one shared response, then run that regression and all-target Clippy successfully. The existing regression supplies the failing test; do not add a duplicate.
- Defers: broad fixture identifiers and replay-server refactoring to the next release.

PM decision: 2026-10-17. Ian approved repairing main before disabling automatic workflows. This ticket restores the fixture behavior removed by the lint repair, without broadening the product change. Before editing code, record the exact baseline command, revision and failure, plus the file claim above. The coordinator owns these records; the developer reports evidence without editing status.
