# 2355 — Recover all-indexed-field variant paper candidates

Status: OPEN.

Milestone: 0.9.4

Reviews: revision 91184cbfd0f29585ebf26689ac18ae9c4c3dbe8c, accept

Reviews: revision e202246e1fd82c109eef793901d98fd3ab638fc5, accept

## Outcome

Variant article discovery retains a bounded Europe PMC all-indexed-field gene plus compatible supplied-allele alias route when title/abstract-only strict work has run. Coverage of restricted fields cannot suppress a non-equivalent all-field route. Recover both clinically confirmed BRCA1 P1749R expert papers while preserving supplied identity, typed annotations, provenance, existing budgets and deadlines. Candidate discovery does not claim verified allele relevance for every lexical hit. No gene-token fallback, schema expansion or guard increase.

- Changes: `src/entities/article/variant_search.rs` compiled all-field plans, route execution and work accounting; `tests/test_variant_article_field_scope.py`, `tests/test_variant_article_alias_coverage.py`, recovery controls and recorded-row fixtures; affected transcript replay controls; `tests/feature-links.json`; `CHANGELOG.md`.
- Proof: a recorded-provider outside-in default variant-articles run recovers PMID10486320 and31788995, which are absent before the fix; show exact all-field versus restricted query coverage, negative identity controls and bounded-work behavior. Inventory all existing replay consumers affected by new requests before review.

## Evidence

- PM m3492 authorizes one 0.9.4 fix per confirmed research cause affecting multiple papers. Parent2350 compares all42 globally unique papers (43 case/paper occurrences,56 rawtraces), with clinical judgment required independently of lexical membership.
- Current-main M5 j4307 on f7165ac3b6e2c25367f31f25393343313564b5e7: default BRCA1 P1749R union returned35 rows after60.039s, complete false, neither expert in candidate_trace. Request/item budgets remained; invocation deadline expired. This timing observation alone does not establish a policy defect or authorize raising limits.
- Current-main M5 j4346: supported `search article -g BRCA1 -k Pro1749Arg --source europepmc --limit 50` returns10486320; the same command with `-k c.5246C>G` returns31788995 and32257056. Corresponding direct all-field alias unions find27/32 rows versus4 for TITLE_ABS-restricted fields; both experts are absent in restricted results. Totals fit one page, and the missing experts never reach default candidates, excluding page size and ranking as these two causes.
- Mechanical trace: variant_search includes_boolean_alias_sources suppresses Europe PMC exact_lexical routes after strict Boolean alias coverage, despite different field scopes; normalized coding/three-letter recovery also remains restricted. Fix the demonstrated missing-field discovery, not just deadline allocation. Existing all-field CLI path proves no new interface is required.
- All42 study identifiers reappear in their corresponding current unrestricted queries and none in the corresponding restricted replies; broad gene-only/date queries remain non-equivalent variant searches and their lexical hits are not asserted same-allele papers. Root clinical decision confirms only the two named expert papers here, with versioned registry evidence.
- Yellow ticket-evidence/2350/first-probe and field-comparison retain exact commands, source responses and assertions. Preserve failed comparison attempts that embedded provider DSL in provider-neutral keywords as non-equivalent probes, not proof.

- Actual authored-row default CLI RED j4363 failed on missing both experts while request budget remained. Initial j4361 failed fixture selection before product assertions; j4366 exposed a wrong optional identity-field assumption and new request consumers. Shared transcript/status controls now explicitly author the additional empty request cases; historical captures remain unchanged.
- Native generated-query proof j4390 returned HTTP200/application-json with31 hits but only25 rows:31788995 at row25,10486320 absent. The early selected-row overlay did not prove real pagination recovery and is superseded. Preserve this failed semantic proof. A separate general supplied-compatible long-protein priority plan keeps all25-row/request and existing work/deadline limits. Exact priority proof j4397 returned HTTP200/application-json and10486320 alone, body SHA2563918ee573cec04816d8d7a456d1c92882b81961485c2f0ec215dfdcd70428a56; unchanged union body SHA256cdd42db62dde1de9e68c2ce93d1a4f2859eb31b693bf478bf9bca5ebfd3037f7 proves31788995 on its first page. No further live requests are needed.
- Final regression uses both actual query-matching bodies and first-page membership, preserving the union's truthful31-total/25-row truncation. It no longer inserts both papers into a substring-matched positive overlay. Both planned units retain exact_lexical provenance and budget accounting after the first strict precision query; all prior selected queries and lexical/identity allowances remain. PubMed and typed identity are unchanged.
- Final affected j4400 passed25 Python cases and66 Rust cases, Ruff/format and1050 feature links; j4397 Clippy passed identical product bytes. An attempted duplicate routine j4377 was stopped and ended143; it is not a completed routine pass. The generic per-developer routine instruction yields to the user's affected-only/one-batch-gate policy. Other fixture, transport, line-ceiling and exact alias-key correction attempts remain identified in the evidence.
- Fresh reviewer accepts91184cbfd0f29585ebf26689ac18ae9c4c3dbe8c after checking exact receipts, identity/provenance/limits/shared-plan accounting and the corrected page-boundary regression. Independent built merged-candidate j4405 passed the actual outside-in recovery case with both experts and explicit truncation. No reviewer edits. Latest main042917b26 is merged; final integration gate belongs to the lander.
