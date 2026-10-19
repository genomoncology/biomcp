---
priority: 4
---

# 2236 — Finish the remaining replay command coverage

Status: OPEN.

Scheduling: Outside 0.9.4; assign the later milestone declared by PM before starting a lane.

## Outcome

Extend the existing recorded replay and JSON/Markdown goldens to the remaining command families after the narrowed 2079 and 2111 release batches. Keep authored fault controls and excluded local datasets in their existing helpers. Each implementation batch declares its own exact source claim before code changes.

## Inventory

The completed coverage includes search all, gene, trial and pathway, and get gene, variant, trial and pathway. The remaining search selectors are author, disease, diagnostic, pgx, phenotype, gwas, article, variant, drug, cell-line, protein and adverse-event. Remaining get selectors are author, article, disease, diagnostic, pgx, drug, cell-line, protein and adverse-event. Also inventory traversal commands and local-data outputs; distinguish byte or JSON-only outputs from commands with both JSON and Markdown forms.

Group related commands by their capture dependencies. Protein identity and structures can reuse the recorded UniProt P15056 identity reply. Protein search needs a fresh exact request because the old receipted query omits the fields parameter production sends. Article graph recommendations and its seed batch have receipts; citations and references need exact requests including the current offset=0. Group article content, fulltext, assets and annotation commands into one dependency audit rather than one gate per example. The old Europe PMC paper request used page=1 while current production uses cursorMark=*; those URLs are not interchangeable. Only the Tables.xls supplement challenge has a disclosed receipted reply; audit other supplement requests individually. Full=true PubTator exports do not substitute for ordinary exports. Gene optional sections have QuickGO, STRING, HPA, DGIdb, NIH RePORTER, MyGene and Open Targets inputs, but exact current requests must be inventoried together.

## Keeps

Preserve existing complete or explicitly minimized source captures and their receipts. Mismatched request receipts require fresh exact captures, never silent URL remapping. Preserve the ComplexPortal participant-filtering and request controls, CTGov alias/deduplication/detail-reuse/failure cases, authored article failure and asset scenarios, provider adversarial cases and local dataset provisioning. A successful real recording cannot replace an authored fault control unchanged. Retire a helper only after its final retained consumer is accounted for.

## Proof

Each cohesive batch adds executed output goldens that reject changed output and passes the affected recorded consumer checks without missing requests. Keep original reader claims and unique request/refusal assertions. Reuse existing replay lifecycle, runner and measurement tools; add no framework. Record exactly which command selectors and traversal outputs are covered, then close this follow-up only at its approved finite boundary.

PM decision: 2026-10-19. This follow-up is outside 0.9.4. That release ends replay work after the accepted gene and pathway goldens and supported pathway reader migration; authored fault controls and excluded local datasets remain in their helpers.
