# 2355 — Recover all-indexed-field variant paper candidates

Status: OPEN.

Milestone: 0.9.4

## Outcome

Variant article discovery retains a bounded Europe PMC all-indexed-field gene plus compatible supplied-allele alias route when title/abstract-only strict work has run. Coverage of restricted fields cannot suppress a non-equivalent all-field route. Recover both clinically confirmed BRCA1 P1749R expert papers while preserving supplied identity, typed annotations, provenance, existing budgets and deadlines. Candidate discovery does not claim verified allele relevance for every lexical hit. No gene-token fallback, schema expansion or guard increase.

- Changes: `src/entities/article/variant_search.rs` compiled all-field plans, route execution and work accounting; `tests/test_variant_article_field_scope.py`, recovery controls and recorded-row fixtures; affected transcript replay controls; `tests/feature-links.json`; `CHANGELOG.md`.
- Proof: a recorded-provider outside-in default variant-articles run recovers PMID10486320 and31788995, which are absent before the fix; show exact all-field versus restricted query coverage, negative identity controls and bounded-work behavior. Inventory all existing replay consumers affected by new requests before review.

## Evidence

- PM m3492 authorizes one 0.9.4 fix per confirmed research cause affecting multiple papers. Parent2350 compares all42 globally unique papers (43 case/paper occurrences,56 rawtraces), with clinical judgment required independently of lexical membership.
- Current-main M5 j4307 on f7165ac3b6e2c25367f31f25393343313564b5e7: default BRCA1 P1749R union returned35 rows after60.039s, complete false, neither expert in candidate_trace. Request/item budgets remained; invocation deadline expired. This timing observation alone does not establish a policy defect or authorize raising limits.
- Current-main M5 j4346: supported `search article -g BRCA1 -k Pro1749Arg --source europepmc --limit 50` returns10486320; the same command with `-k c.5246C>G` returns31788995 and32257056. Corresponding direct all-field alias unions find27/32 rows versus4 for TITLE_ABS-restricted fields; both experts are absent in restricted results. Totals fit one page, and the missing experts never reach default candidates, excluding page size and ranking as these two causes.
- Mechanical trace: variant_search includes_boolean_alias_sources suppresses Europe PMC exact_lexical routes after strict Boolean alias coverage, despite different field scopes; normalized coding/three-letter recovery also remains restricted. Fix the demonstrated missing-field discovery, not just deadline allocation. Existing all-field CLI path proves no new interface is required.
- All42 study identifiers reappear in their corresponding current unrestricted queries and none in the corresponding restricted replies; broad gene-only/date queries remain non-equivalent variant searches and their lexical hits are not asserted same-allele papers. Root clinical decision confirms only the two named expert papers here, with versioned registry evidence.
- Yellow ticket-evidence/2350/first-probe and field-comparison retain exact commands, source responses and assertions. Preserve failed comparison attempts that embedded provider DSL in provider-neutral keywords as non-equivalent probes, not proof.
