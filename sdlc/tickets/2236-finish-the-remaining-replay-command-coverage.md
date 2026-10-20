---
priority: 4
---

# 2236 — Finish the remaining replay command coverage

Status: OPEN.

Milestone: 0.9.4

Depends on: 2287

Reviews: revision e44b173, accept

Reviews: revision e3c49e8, accept

Reviews: revision 61c64f5, accept

## Outcome

Extend the existing recorded replay and JSON/Markdown goldens to the remaining command families after the narrowed 2079 and 2111 release batches. Keep authored fault controls and excluded local datasets in their existing helpers. Each implementation batch declares its own exact source claim before code changes.

Changes: `tests/golden/test_golden_output.py`, `tests/golden/cases/get-protein-p15056.*`, `tests/golden/cases/get-protein-p15056-structures.*`, `tests/golden/cases/protein-structures-p15056-limit1-offset1.*`, `tests/golden/cases/search-disease-melanoma-limit1.*`, `tests/golden/cases/search-variant-braf-melanoma-limit3.*`, `tests/golden/cases/search-gwas-braf-limit49.*`, `tests/golden/cases/search-gwas-melanoma-limit49.*`, `tests/golden/cases/search-pgx-cyp2d6-limit20.*`, `tests/golden/cases/search-pgx-clopidogrel-limit20.*`, `tests/golden/cases/get-author-orcid-4299-3018.*`, `tests/golden/cases/author-papers-orcid-4299-3018-limit3.*`, `tests/golden/cases/search-protein-braf-limit3.*`, `tests/golden/cases/search-phenotype-hp1250-hp1263-limit1.*`, `tests/golden/cases/get-pgx-cyp2d6-interactions.*`, `tests/golden/cases/get-pgx-cyp2d6-recommendations.*`, `testdata/replay/manifest.json`, `testdata/replay/responses/cpic/*`, `testdata/replay/responses/orcid/*`, `testdata/replay/responses/uniprot/*`, `testdata/replay/responses/hpo/*`, `testdata/replay/responses/monarch/*`, `sdlc/records/2236-recorded-command-goldens.md`, `CHANGELOG.md`.

Additional third-batch Changes: `tests/golden/cases/get-cell-line-*`, `tests/golden/cases/search-cell-line-*`, `tests/golden/cases/get-gene-braf-go-interactions-hpa.*`, `tests/golden/cases/get-gene-egfr-druggability.*`, `tests/golden/cases/get-gene-erbb2-funding.*`, `tests/golden/cases/get-article-*`, `tests/golden/cases/article-entities-*`, `testdata/replay/responses/cellosaurus/*`, `testdata/replay/responses/mygene/*`, `testdata/replay/responses/quickgo/*`, `testdata/replay/responses/string/*`, `testdata/replay/responses/hpa/*`, `testdata/replay/responses/dgidb/*`, `testdata/replay/responses/opentargets/*`, `testdata/replay/responses/nih-reporter/*`, `testdata/replay/responses/pubtator/*`, `testdata/replay/responses/europepmc/*`, `testdata/replay/responses/pubmed/*`, `testdata/replay/responses/mydisease/*`.

Final OA batch Changes: `tests/golden/replay_env.py`, `tests/golden/test_golden_output.py`, `tests/golden/cases/get-article-23056577-fulltext.*`, `tests/golden/cases/get-article-23056577-assets.json`, `testdata/replay/manifest.json`, `testdata/replay/responses/pmc-oa/*`, `testdata/replay/responses/pmc-html/*`, `testdata/replay/responses/pubtator/*`, `testdata/replay/responses/europepmc/*`, `testdata/replay/responses/pubmed/*`, `testdata/replay/responses/mydisease/*`, the existing command-golden record, and `CHANGELOG.md`.

## Inventory

The completed coverage includes search all, gene, trial and pathway, and get gene, variant, trial and pathway. The remaining search selectors are author, disease, diagnostic, pgx, phenotype, gwas, article, variant, drug, cell-line, protein and adverse-event. Remaining get selectors are author, article, disease, diagnostic, pgx, drug, cell-line, protein and adverse-event. Also inventory traversal commands and local-data outputs; distinguish byte or JSON-only outputs from commands with both JSON and Markdown forms.

Group related commands by their capture dependencies. Protein identity and structures can reuse the recorded UniProt P15056 identity reply. Protein search needs a fresh exact request because the old receipted query omits the fields parameter production sends. Article graph recommendations and its seed batch have receipts; citations and references need exact requests including the current offset=0. Group article content, fulltext, assets and annotation commands into one dependency audit rather than one gate per example. The old Europe PMC paper request used page=1 while current production uses cursorMark=*; those URLs are not interchangeable. Only the Tables.xls supplement challenge has a disclosed receipted reply; audit other supplement requests individually. Full=true PubTator exports do not substitute for ordinary exports. Gene optional sections have QuickGO, STRING, HPA, DGIdb, NIH RePORTER, MyGene and Open Targets inputs, but exact current requests must be inventoried together.

## Keeps

Preserve existing complete or explicitly minimized source captures and their receipts. Mismatched request receipts require fresh exact captures, never silent URL remapping. Preserve the ComplexPortal participant-filtering and request controls, CTGov alias/deduplication/detail-reuse/failure cases, authored article failure and asset scenarios, provider adversarial cases and local dataset provisioning. A successful real recording cannot replace an authored fault control unchanged. Retire a helper only after its final retained consumer is accounted for.

## Proof

Each cohesive batch adds executed output goldens that reject changed output and passes the affected recorded consumer checks without missing requests. Keep original reader claims and unique request/refusal assertions. Reuse existing replay lifecycle, runner and measurement tools; add no framework. Record exactly which command selectors and traversal outputs are covered, then close this follow-up only at its approved finite boundary.

PM decision: 2026-10-20. Ian moved this coverage into 0.9.4; it already found the protein structures JSON panic (2287). Land it after the release-blocking repairs, in cohesive batches.

## Progress

- 2026-10-20 started

## First batch boundary

Use the existing exact UniProt P15056 recording to cover get protein identity, get protein structures and protein structures with limit 1 and offset 1 in both JSON and Markdown. Existing captures and authored fault controls remain unchanged. This pagination selects the second of the two retained structures instead of testing only an empty page. Protein search and every other selector stay outside this first implementation batch. The 2287 repair owns the reproduced JSON panic. This batch cannot close the whole follow-up; inventory the remaining exact captures and missing inputs before proposing further batches.


Keep: experiments/2102-biomcp-2258-final-live-replay — retain protein-goldens/2236 raw preimage and corrected job evidence while release admission consumes this batch.

PM completion boundary, 2026-10-20: cover protein, every existing exact or receipted recording group, and the finite approved public capture plan. Semantic Scholar, KEGG, restricted and local sources stay excluded. The current first batch is partial and cannot close this parent. Exact request-count and no-Semantic-Scholar clarification is required before new captures.

## Second batch boundary

Starting from 87df265ba55d14fd5f8ae66b969454dd351f8b67, extend the existing golden table with disease and variant search, GWAS gene and trait search, PGx gene and drug search, ORCID identity and claimed works, protein search, phenotype matching, and the two successful CYP2D6 interaction/recommendation sections. Reuse canonical routes and unchanged exact receipted CPIC/ORCID replies; import approved current successful public captures without changing any request. The existing absent-golden assertion is the failing outside-in check before update mode generates the expected output. Keep all accepted protein cases and authored controls. CPIC frequency and guideline reads returned real HTTP400 schema errors and cannot establish successful section coverage; retain raw errors and report the product defects separately. No product code or replay helper changes.

## Third batch boundary

Extend the existing golden table with exact receipted cell-line identity/search, gene GO/interactions/HPA/druggability/funding, and ordinary article identity/annotations/entities. Audit disease detail dependency closure before including it. Preserve all accepted cases and existing authored controls. Semantic Scholar stays explicitly disabled on article cases and its graph/recommendation routes remain excluded. Reuse each original receipt only with its matching request. The current NIH funding capture supplies its original request body. No runtime or replay helper changes are proposed.

Changes for this batch: `tests/golden/test_golden_output.py`, corresponding new `tests/golden/cases/get-cell-line-*`, `tests/golden/cases/search-cell-line-*`, `tests/golden/cases/get-gene-braf-go-interactions-hpa.*`, `tests/golden/cases/get-gene-egfr-druggability.*`, `tests/golden/cases/get-gene-erbb2-funding.*`, `tests/golden/cases/get-article-*`, `tests/golden/cases/article-entities-*`, optional `tests/golden/cases/get-disease-*`, `testdata/replay/manifest.json`, `testdata/replay/responses/cellosaurus/*`, `testdata/replay/responses/mygene/*`, `testdata/replay/responses/quickgo/*`, `testdata/replay/responses/string/*`, `testdata/replay/responses/hpa/*`, `testdata/replay/responses/dgidb/*`, `testdata/replay/responses/opentargets/*`, `testdata/replay/responses/nih-reporter/*`, `testdata/replay/responses/pubtator/*`, the existing command-golden record, and `CHANGELOG.md`. Existing absent-golden assertions provide the outside-in red before generating outputs; the green must replay without misses.

Third-batch dependency claim: `testdata/replay/responses/europepmc/*`, `testdata/replay/responses/pubmed/*`, and `testdata/replay/responses/mydisease/*` supply the exact identity verification and receipted disease-crosswalk dependencies identified by the request probe. The PM approved these public keyless normal-route captures before calls.

## Final OA article boundary

Use the clinically selected PMC3467229/PMID23056577 with its own current metadata proving license_code CC BY and is_pmc_openaccess true. Add fulltext JSON/Markdown, JSON-only assets manifest and one exact byte retrieval of supplementary PRISMA checklist pone.0047054.s002.doc. Existing asset helpers retain challenge, licensing, binary and local-dataset controls. Keep Semantic Scholar and Figshare discovery disabled in these cases. No new bodies for PMC3040717 or PMC9984800. Capture only finite observed normal-route requests, retaining metadata and one transient retry. Report additional links before calls. Extend existing Case with default formats and ReplayEnvironment.run with default text=True; normalize only the current run's literal isolated-cache prefix, preserving filename and content. No new framework.
