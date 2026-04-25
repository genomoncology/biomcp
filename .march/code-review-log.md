# Code Review Log — Ticket 302

## Phase 1 — Critique

- Audited `.march/design-final.md` against the branch diff and confirmed the ticketed contract surfaces were updated before runtime changes: `spec/surface/discover.md`, `src/cli/commands.rs`, `src/cli/list.rs`, `src/cli/list_reference.md`, `docs/user-guide/discover.md`, `docs/user-guide/cli-reference.md`, `docs/reference/quick-reference.md`, `docs/index.md`, `skills/SKILL.md`, and `architecture/ux/cli-reference.md`.
- Traced the proof matrix to concrete tests:
  - relational redirect/noise suppression: `src/entities/discover.rs` relational warfarin + MEF2 tests and `spec/surface/discover.md`
  - single-entity stability: `exact_gene_query_promotes_hgnc_result`, `single_entity_gene_alias_queries_stay_stable_after_general_filtering`, `single_entity_disease_queries_stay_stable_after_general_filtering`, plus the ERBB1 spec section
  - supported routed flows: `developmental delay`, `chest pain`, `symptoms of Marfan syndrome`, `what drugs treat myasthenia gravis`, `BRAF melanoma`, and `CTCF cohesin` tests
  - JSON redirect contract: `src/render/json.rs::to_discover_json_keeps_relational_redirect_commands_only_under_meta`
- Checked the implementation for security, duplication, and general quality concerns. The new redirect command stays shell-safe by reusing the existing quoting path, and the discover-local relevance helpers remain appropriately local rather than duplicating the article query cleanup helpers.
- Defect found: the deterministic relational unit fixtures did not include the weak HP symptom noise that live OLS can return ahead of the general concepts. That meant the unit proof would not catch a regression where `SymptomSearch` preempted the general relational redirect path; the executable spec exposed this gap.

## Phase 2 — Fix Plan

1. Strengthen the relational discover unit fixtures so they include the same weak symptom contamination seen in live OLS output, keeping the general-intent redirect guard pinned by deterministic proof.

## Phase 3 — Repair

- Updated the warfarin and MEF2 relational unit tests in `src/entities/discover.rs` to include weak HP symptom hits that sort ahead of the general results in live OLS responses.
- Re-ran the targeted discover unit/spec proof, the ticket-owned docs contract tests, and the repo focused validation tier.
- Post-fix collateral scan found no dead code, unused imports, stale error messages, resource cleanup conflicts, or shadowed variables from the repair.

## Residual Concerns

- None.

## Defect Register

| # | Category | Lintable | Description |
|---|----------|----------|-------------|
| 1 | weak-assertion | no | Relational discover unit fixtures omitted weak symptom OLS noise, so they would not catch regressions where `SymptomSearch` preempted the general relational redirect. |
