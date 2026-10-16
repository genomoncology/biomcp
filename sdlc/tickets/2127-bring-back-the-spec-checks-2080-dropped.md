# 2127 — Bring back the spec checks 2080 dropped

Status: OPEN.

Milestone: 0.9.3

Reviews: revision 36302bc, accept

## Outcome

Every claim the 35 spec pages 2080 removed from the routine list used to check runs again before release, either as a routine spec page on the replay server or as a named surviving test, and the release gate runs them.

## Evidence

Found by the 0.9.3 adversarial review on 2026-10-16, against 2080 (landed 6ad795db1).

- Before 2080, `scripts/run-specs.sh` listed 37 routine pages (6ad795db1^1, lines 13-51). Now it lists `cli-contract-ratchet.md`, `build-profile.md` and `tests/golden/spec`, which holds four tests (gene, variant, two search-all). `make spec`, `spec-pr`, `spec-contracts`, CI (`ci.yml:251`) and `release-gate` all read that list.
- The article, disease, drug, gene, trial, variant, pgx, section-outcomes and mcp pages run nowhere; their executable blocks were deleted and the prose kept (for example `spec/surface/mcp.md:242-306`).
- The end-to-end check that raw MCP returns the same diagnostic provenance as the CLI is gone; only unit tests remain.
- 2080's handoff names a page that does not exist ("clingen-cspot") and leaves out `spec/surface/trial-retirement.md`, which still exists and is unlisted.

- Starts from: 6ad795db1^1's routine list, `scripts/run-specs.sh`, `tests/golden/`, the 2079 replay server.
- Decisions:
  - For each of the 35 pages, write one line in this ticket: re-listed on the replay server, or covered by a named surviving test, or deleted with the reason. A page claim with neither keeps its page.
  - Re-listed pages run in the routine spec lane within 2080's time target; a page that cannot meet it runs in the release gate instead and says so.
  - 2111 keeps the fixture-consumer moves; this ticket owns the spec list.
- Changes: claims `scripts/run-specs.sh` `spec/**` `tests/golden/**` `testdata/replay/**` `CHANGELOG.md`.
- Proof: the 35-line table in this ticket; `make spec` and the release gate run every re-listed page green on the replay server; the spec lane time recorded against 2080's target.
- Order: before release; after 2079's replay server (landed).

## The 35-page table (2127 build, 2026-10-17)

One line per page the 2080 handoff dropped. "Re-listed" pages run in
`scripts/run-specs.sh`'s routine list at the branch tip this table records.
"Keeps its page" pages stay on disk unlisted, as the handoff left them: their
recordings are not authored and no surviving test owns their whole claim set;
2079's later conversion batches pick them up.

| # | Page | Disposition | Evidence / reason |
|---|---|---|---|
| 1 | spec/surface/skills.md | Re-listed, routine, replay server | Every block is a local skill install/status/list check; no provider traffic; page unchanged. |
| 2 | spec/entity/trial-numeric-filters.md | Re-listed, routine, replay server | Block rewired from the deleted CTGov fixture log to the lane's shared `$BIOMCP_REPLAY_REQUEST_LOG`; the rejected radius still makes zero provider requests. |
| 3 | spec/surface/trial-retirement.md | Re-listed, routine, replay server | Refusal, help and docs blocks are local; the literal-condition block now runs on the replay server against a hand-authored Phelan-McDermid recording and asserts the replay request log. |
| 4 | spec/entity/clingen-erepo.md | Re-listed, routine, frozen-capture runner | Replays its own frozen capture through `spec/fixtures/run-clingen-erepo-fixture.sh` over `testdata/sources/clingen_erepo`, not tools/replay-server (coordinator ruling, 2127). Parser facts also held by unit tests in `src/sources/clingen_erepo.rs`. |
| 5 | spec/entity/variant-articles-corpus.md | Re-listed, routine | Replays its frozen corpus through `spec/fixtures/run-variant-articles-corpus-canary.sh` over `testdata/sources` captures, not tools/replay-server (coordinator ruling, 2127). The routine lane with this page included runs inside 2080's 1.0-1.3 min target, so it stays routine; final green pass at the reviewed tip `1e968f3b6`: the CI spec-lane's `make spec` step ran 79 seconds (run 38959612104). |
| 6 | spec/entity/article.md | Keeps its page | Runnable blocks read the article fulltext fixture's request log and runner; Europe PMC/PubMed recordings exist only for the golden-case commands. |
| 7 | spec/entity/article-text-fidelity.md | Keeps its page | Needs the article fulltext fixture's JATS/HTML captures (2111 owns that fixture's consumers) or new recordings; none authored. |
| 8 | spec/entity/author.md | Keeps its page | Semantic Scholar/ORCID recordings not authored; the existing s2 recordings cover only the golden-case shapes. |
| 9 | spec/entity/disease.md | Keeps its page | Blocks count traffic through the deleted disease-survival and provider-contract fixture logs; recordings not authored. |
| 10 | spec/entity/disease-survival-fixture.md | Keeps its page | The fixture script it names was deleted in 2080; conversion needs MyDisease CML plus SEER Explorer recordings; none authored. |
| 11 | spec/entity/phenotype.md | Keeps its page | Reads the deleted disease-survival fixture's readiness file and request log; HPO recordings not authored. |
| 12 | spec/surface/discover.md | Keeps its page | Asserts OLS4 request shapes through the deleted disease-survival fixture's request log; OLS4 recordings not authored. |
| 13 | spec/entity/diagnostic.md | Keeps its page | GTR/WHO-IVD/OpenFDA/MyDisease rows came from the deleted provider-contract and disease-survival fixtures; recordings not authored. |
| 14 | spec/entity/vaers.md | Keeps its page | Partial: `tests/adverse_event_route_contract.rs` (`invalid_count_offset_is_rejected_without_provider_contact`, `combined_faers_filter_keeps_vaers_visibly_not_requested`) holds the FAERS count-field and combined-source claims; the aggregate contract blocks need openFDA recordings. |
| 15 | spec/entity/cell-line.md | Keeps its page | Cellosaurus recordings not authored. |
| 16 | spec/entity/pathway.md | Keeps its page | Provider-contract request-log assertions plus KEGG/Reactome/WikiPathways recordings beyond the golden-case shapes; none authored. |
| 17 | spec/entity/trial.md | Keeps its page | Partial: the trial leg's query shape is held by `tests/golden/spec/test_spec_page_golden.py::test_search_all_consults_the_recorded_provider_matrix`; the page's detail and log-arithmetic blocks need their own recordings. |
| 18 | spec/entity/drug-label-sections.md | Keeps its page | Provider-contract log plus openFDA recordings not authored. |
| 19 | spec/entity/drug.md | Keeps its page | EMA/WHO dataset directories and provider-contract log; MyChem/openFDA recordings beyond golden shapes not authored. |
| 20 | spec/entity/gene.md | Keeps its page | Partial: acquisition arithmetic is held by `tests/golden/spec/test_spec_page_golden.py::test_get_gene_queries_mygene_once_by_symbol`; identity, local-filter and pagination blocks assert fixture data (ODC1, FILTERCASE) needing authored MyGene recordings. |
| 21 | spec/entity/drug-interactions.md | Keeps its page | Reads DDInter from a local bundle the deleted setup script installed; a committed bundle or recordings not authored. |
| 22 | spec/entity/pgx.md | Keeps its page | CPIC recordings for the CYP2D6/clopidogrel commands and the recommendation/guideline/frequency sections not authored. |
| 23 | spec/entity/section-outcomes.md | Keeps its page | Blocks switch DDInter bundle directories through fixture env the deleted setup script supplied. |
| 24 | spec/entity/study.md | Keeps its page | Reads cBioPortal studies from `BIOMCP_STUDY_DIR` the deleted setup script populated; recordings not authored. |
| 25 | spec/entity/trial-intervention-aliases.md | Keeps its page | Blocks call the CTGov intervention-alias fixture scripts directly; conversion pending (2111 owns that fixture's test consumers). |
| 26 | spec/entity/trial-documents.md | Keeps its page | CTGov CDN large-docs recordings not authored. |
| 27 | spec/entity/variant.md | Keeps its page | Partial: protein-change query arithmetic is held by `tests/golden/spec/test_spec_page_golden.py::test_get_variant_queries_myvariant_once_by_protein_change`; the remaining blocks need MyVariant/UniProt/InterPro/CancerHotspots recordings. |
| 28 | spec/entity/variant-input-forms.md | Keeps its page | MyVariant/Mutalyzer/VariantValidator/ClinGen-AR recordings for its five identity forms not authored. |
| 29 | spec/entity/variant-protein-change-resolution.md | Keeps its page | UniProt recordings not authored. |
| 30 | spec/entity/variant-gene-first-routing.md | Keeps its page | MyGene/MyVariant recordings for its routing table not authored. |
| 31 | spec/entity/clingen-cspec.md | Keeps its page | Its runner needs a local CSpec origin and request counter that the fixture setup script deleted in 2080 supplied; an erepo-style runner rewrite or cspec recordings remain the conversion gap. |
| 32 | spec/entity/variant-article-identity.md | Keeps its page | Runner `spec/fixtures/run-variant-article-identity-fixture.sh` is self-contained like erepo's, so re-listing needs only the same mechanism ruling and a timing measure; not attempted in this budget. |
| 33 | spec/entity/article-entities.md | Keeps its page | PubTator/Europe PMC entity recordings beyond the golden-case shapes not authored. |
| 34 | spec/surface/mcp.md | Keeps its page | The dropped end-to-end raw-MCP-versus-CLI provenance check is not covered; typed parity is partly held by `src/mcp/shell/typed_get_tests.rs`. |
| 35 | spec/entity/protein.md | Keeps its page | Reads the ComplexPortal fixture's request log (2111 owns that fixture's test consumers) and needs UniProt recordings; none authored. |

The handoff's "clingen-cspot" names no existing page; the count above keeps the
35 real dropped pages, including `spec/surface/trial-retirement.md`, which the
handoff left out and line 3 resolves.
