# Data Sources

BioMCP unifies multiple biomedical data providers behind one CLI grammar.
This reference explains source provenance, authentication requirements, base endpoints,
and operational caveats so users can reason about result quality and troubleshooting.
Use [Source Licensing and Terms](source-licensing.md) for provider terms, reuse constraints, and indirect-only provenance rows.

## Source matrix

| Entity / feature | Primary source(s) | Base URL | Auth required | Notes |
|------------------|-------------------|----------|---------------|-------|
| Gene | MyGene.info | `https://mygene.info/v3` | No | Symbol lookup, aliases, summaries |
| Gene sections | UniProt, QuickGO, STRING, GTEx, Human Protein Atlas, DGIdb, OpenTargets, ClinGen, gnomAD GraphQL API | `https://rest.uniprot.org`, `https://www.ebi.ac.uk/QuickGO/services`, `https://string-db.org/api`, `https://gtexportal.org/api/v2`, `https://www.proteinatlas.org`, `https://dgidb.org/api/graphql`, `https://api.platform.opentargets.org/api/v4/graphql`, `https://search.clinicalgenome.org`, `https://gnomad.broadinstitute.org/api` | No | Protein summary, GO terms, interactions, GTEx RNA tissue expression, HPA protein tissue expression and subcellular localization, combined DGIdb/OpenTargets druggability, gene-disease validity, and gnomAD v4 GRCh38 gene constraint |
| Gene `disgenet` section | DisGeNET REST API | `https://api.disgenet.com/api/v1` | Yes (`DISGENET_API_KEY`) | Ranked scored gene-disease associations with PMIDs, clinical-trial counts, evidence index, and evidence level |
| Variant | MyVariant.info | `https://myvariant.info/v1` | No | rsID/HGVS lookup, ClinVar and population annotations |
| Variant population section | MyVariant.info (gnomAD fields) | `https://myvariant.info/v1` | No | Uses cached gnomAD AF/subpopulation fields from MyVariant payload |
| Variant GWAS section and GWAS search | GWAS Catalog REST API | `https://www.ebi.ac.uk/gwas/rest/api` | No | rsID, gene, and trait association retrieval |
| Variant OncoKB helper | OncoKB | `https://www.oncokb.org/api/v1` | Yes (`ONCOKB_TOKEN`) | Accessed via explicit `variant oncokb <id>` command |
| Variant prediction | AlphaGenome | `https://gdmscience.googleapis.com:443` | Yes (`ALPHAGENOME_API_KEY`) | gRPC scoring for `predict` section |
| Trial (default) | ClinicalTrials.gov API v2 | `https://clinicaltrials.gov/api/v2` | No | Default trial search/get source |
| Trial (optional) | NCI CTS API | `https://clinicaltrialsapi.cancer.gov/api/v2` | Yes (`NCI_API_KEY`) | Enabled via `--source nci` |
| NCI CTS trial search | NCI CTS API | `https://clinicaltrialsapi.cancer.gov/api/v2` | Yes (`NCI_API_KEY`) | `search trial --source nci` |
| Article search & metadata | PubTator3 + Europe PMC + PubMed + LitSense2 + optional Semantic Scholar | `https://www.ncbi.nlm.nih.gov/research/pubtator3-api`, `https://www.ebi.ac.uk/europepmc/webservices/rest`, `https://eutils.ncbi.nlm.nih.gov/entrez/eutils`, `https://www.ncbi.nlm.nih.gov/research/litsense2-api/api`, `https://api.semanticscholar.org` | Optional (`S2_API_KEY`) | Federated search with identifier-aware merge, per-source capping after deduplication and before ranking, plus lexical, semantic, or weighted hybrid relevance ranking |
| Article enrichment and graph helpers | Semantic Scholar | `https://api.semanticscholar.org` | Optional (`S2_API_KEY`) | Search-leg metadata, TLDR, influential citations, citation/reference graph, recommendations |
| Article annotations | PubTator3 | `https://www.ncbi.nlm.nih.gov/research/pubtator3-api` | No | Entity annotations |
| Article full-text resolution | Europe PMC + NCBI E-utilities + PMC OA + NCBI ID Converter + PMC HTML + opt-in Semantic Scholar PDF metadata | `https://www.ebi.ac.uk/europepmc/webservices/rest`, `https://eutils.ncbi.nlm.nih.gov/entrez/eutils`, `https://www.ncbi.nlm.nih.gov/pmc/utils/oa/oa.fcgi`, `https://pmc.ncbi.nlm.nih.gov/tools/idconv/api/v1/articles`, `https://pmc.ncbi.nlm.nih.gov/articles`, `https://api.semanticscholar.org` | Optional (`NCBI_API_KEY`, `S2_API_KEY`) | NCBI ID Converter bridges identifiers; XML/HTML/PDF content rungs save Markdown when available |
| Drug | MyChem.info | `https://mychem.info/v1` | No | Drug metadata, targets, synonyms, and default U.S. search/get normalization |
| Drug EU regional context | EMA website JSON batch (local human-medicines download) | `https://www.ema.europa.eu/en/about-us/about-website/download-website-data-json-data-format` | No | Supports canonical `search/get drug --region eu|all` for regulatory, safety, and shortage, accepts `ema` as an input alias for `eu`, and auto-downloads into `BIOMCP_EMA_DIR` or the platform data directory on first use; `biomcp ema sync` force-refreshes the local files and omitting `--region` on `get drug <name> regulatory` checks U.S. and EU regulatory data |
| Drug WHO regional context | WHO finished-pharmaceutical-products CSV + WHO active-pharmaceutical-ingredients CSV + WHO vaccine CSV (local downloads) | `https://extranet.who.int/prequal/medicines/prequalified/finished-pharmaceutical-products/export?page&_format=csv`, `https://extranet.who.int/prequal/medicines/prequalified/active-pharmaceutical-ingredients/export?page&_format=csv`, `https://extranet.who.int/prequal/vaccines/prequalified/export` | No | Supports `search/get drug --region who|all`, WHO-filtered structured `search drug --region who` for finished-pharma/API, and WHO-only `--product-type <finished_pharma|api|vaccine>` filters; WHO vaccine support is explicit search-only; auto-downloads all three files into `BIOMCP_WHO_DIR` or the platform data directory on first use and `biomcp who sync` force-refreshes the local exports |
| Drug-drug interactions | DDInter public CSV download bundle (local downloads) | `https://ddinter.scbdd.com/download/` | No | Supports `biomcp drug interactions <name>` plus `get drug <name> interactions`; auto-downloads the eight ATC-sliced DDInter CSVs into `BIOMCP_DDINTER_DIR` or the platform data directory on first use, refreshes stale files after 72 hours, and `biomcp ddinter sync` force-refreshes the local bundle; empty results stay scoped to the current DDInter bundle and do not prove absence of clinical interactions |
| Drug vaccine identity bridge | CDC CVX code set + CDC trade-name map + CDC MVX manufacturer table (local downloads) | `https://www2.cdc.gov/vaccines/iis/iisstandards/downloads/cvx.txt`, `https://www2.cdc.gov/vaccines/iis/iisstandards/downloads/TRADENAME.txt`, `https://www2.cdc.gov/vaccines/iis/iisstandards/downloads/mvx.txt` | No | Supports omitted-`--region` plain-name vaccine search plus explicit `search drug <name> --region eu|all` and explicit WHO vaccine name/brand search (`--region who --product-type vaccine`) when MyChem identity resolution misses; auto-downloads the bundle into `BIOMCP_CVX_DIR` or the platform data directory on first use, refreshes stale files after 30 days, and `biomcp cvx sync` force-refreshes the local CDC data; `--region us` stays outside this path |
| Diagnostic | NCBI Genetic Testing Registry bulk exports + WHO IVD CSV export (local downloads) + optional OpenFDA device overlay | `https://ftp.ncbi.nlm.nih.gov/pub/GTR/data/test_version.gz`, `https://ftp.ncbi.nlm.nih.gov/pub/GTR/data/test_condition_gene.txt`, `https://extranet.who.int/prequal/vitro-diagnostics/prequalified/in-vitro-diagnostics/export?page&_format=csv`, `https://api.fda.gov/device/510k.json`, `https://api.fda.gov/device/pma.json` | Optional (`OPENFDA_API_KEY`) for the FDA overlay only | Supports source-aware `search diagnostic --source <gtr|who-ivd|all>` and `get diagnostic`; auto-downloads the GTR files into `BIOMCP_GTR_DIR`, `who_ivd.csv` into `BIOMCP_WHO_IVD_DIR`, refreshes stale GTR data after 7 days and WHO IVD data after 72 hours, and `biomcp gtr sync` / `biomcp who-ivd sync` force-refresh the local bundles; `get diagnostic <id> regulatory` adds an opt-in live OpenFDA device 510(k)/PMA lookup without changing the base summary card |
| Drug section enrichments | ChEMBL + OpenTargets + CIViC | `https://www.ebi.ac.uk/chembl/api/data`, `https://api.platform.opentargets.org/api/v4/graphql`, `https://civicdb.org/api` | No | Generic targets/mechanisms from ChEMBL, generic target/indication context from Open Targets, and additive CIViC variant-target annotations for drug target output |
| Disease normalization | MyDisease.info | `https://mydisease.info/v1` | No | MONDO-oriented disease normalization |
| Discover structured concepts | OLS4 | `https://www.ebi.ac.uk/ols4` | No | Free-text ontology search for `biomcp discover`; OLS4 is the required backbone |
| Discover clinical crosswalks | UMLS REST API | `https://uts-ws.nlm.nih.gov/rest` | Optional (`UMLS_API_KEY`) | Adds ICD-10, SNOMED CT, RxNorm, OMIM, and related cross-vocabulary IDs to discover results |
| Discover plain-language topics | MedlinePlus Search | `https://wsearch.nlm.nih.gov/ws/query` | No | Best-effort disease/symptom context for `biomcp discover`; suppressed for gene/drug/pathway flows |
| Disease `clinical_features` section | MedlinePlus Search | `https://wsearch.nlm.nih.gov/ws/query` | No | Opt-in clinical-summary feature rows for configured diseases, with embedded reviewed fixtures as the offline fallback |
| Phenotype term resolution | HPO JAX API | `https://ontology.jax.org/api/hp` | No | Direct HPO term lookup and normalization used by phenotype workflows |
| Disease genes/pathways/prevalence | OpenTargets GraphQL + Reactome | `https://api.platform.opentargets.org/api/v4/graphql`, `https://reactome.org/ContentService` | No | Baseline disease context with ranked associated targets; disease `genes` can promote OpenTargets rows directly into the disease-gene table and attach OT score summaries |
| Disease `survival` section | SEER Explorer | `https://seer.cancer.gov/statistics-network/explorer/source/content_writers` | No | Disease survival detail for mapped cancers, surfaced by the explicit `survival` section and preserved in disease `all`; uses live site-catalog resolution plus all-ages / all-races 5-year relative survival by sex, and degrades to stable notes on mapping or availability failures |
| Disease `genes` and `phenotypes` sections | Monarch Initiative API v3 | `https://api-v3.monarchinitiative.org` | No | Core disease associations and phenotype evidence |
| Disease `genes` and `variants` augmentation | CIViC | `https://civicdb.org/api` | No | Somatic driver augmentation for genes and disease-associated molecular profiles |
| Disease `models` section | Monarch Initiative API v3 | `https://api-v3.monarchinitiative.org` | No | Model-organism evidence with relationship and provenance |
| Disease `disgenet` section | DisGeNET REST API | `https://api.disgenet.com/api/v1` | Yes (`DISGENET_API_KEY`) | Ranked scored disease-gene associations; disease lookup uses UMLS-backed DisGeNET identifiers |
| Gene/Disease `funding` section | NIH Reporter v2 API | `https://api.reporter.nih.gov/v2` | No | Exact-phrase title/abstract funding lookup over the most recent 5 NIH fiscal years; returns top unique grants after de-duplicating project-year records |
| Phenotype search (`search phenotype`) | Monarch Initiative API v3 | `https://api-v3.monarchinitiative.org` | No | HPO set similarity search to ranked diseases |
| PGx core interactions/recommendations | CPIC API | `https://api.cpicpgx.org/v1` | No | Pair, recommendation, frequency, and guideline views |
| PGx annotations section | PharmGKB API | `https://api.pharmgkb.org/v1` | No | Clinical/guideline/label annotation enrichment |
| Pathway | Reactome + KEGG + WikiPathways + g:Profiler | `https://reactome.org/ContentService`, `https://rest.kegg.jp`, `https://www.wikipathways.org/json`, `https://biit.cs.ut.ee/gprofiler/api` | No | Pathway search and detail use Reactome + KEGG + WikiPathways; `genes` are available across all three sources, while `events` and pathway `enrichment` remain Reactome-only; top-level `biomcp enrich` uses **g:Profiler** |
| Protein | UniProt + InterPro + STRING + ComplexPortal | `https://rest.uniprot.org`, `https://www.ebi.ac.uk/interpro/api`, `https://string-db.org/api`, `https://www.ebi.ac.uk/intact/complex-ws` | No | Protein cards, domains, interactions, structures, and human protein complex membership; structure IDs are surfaced from UniProt cross-references to PDB and AlphaFold DB |
| Drug/device safety, labels, shortages, approvals, and diagnostic regulatory overlay | OpenFDA | `https://api.fda.gov` | Optional (`OPENFDA_API_KEY`) | FAERS, MAUDE, recalls, drug labels, shortages, Drugs@FDA-derived approvals, and exact-name-first diagnostic device 510(k)/PMA overlays |
| Vaccine adverse-event search | CDC WONDER VAERS | `https://wonder.cdc.gov/controller/datarequest/D8` | No | Aggregate-only vaccine adverse-event summaries for `search adverse-event --source vaers|all`; BioMCP uses the CDC WONDER XML POST contract, includes the required data-use agreement, and resolves vaccine identity through the CDC CVX/MVX bridge when available |
| Gene enrichment sections | Enrichr | `https://maayanlab.cloud/Enrichr` | No | Gene enrichment sections inside entity outputs use Enrichr; this is distinct from top-level `biomcp enrich` |
| Cohort frequencies (best-effort) | cBioPortal | `https://www.cbioportal.org/api` | No | Supplemental cancer frequency context |

## Global HTTP behavior

All HTTP-based sources share a common client with:

- Connect timeout: 10 seconds
- Request timeout: 30 seconds
- Retries: exponential backoff, up to 3 retries for transient failures
- Disk cache: `<cache_root>/http` under the resolved cache root (`~/.cache/biomcp/http` on Linux)

cBioPortal DataHub study archive downloads are the exception: archive downloads do
not use a total request timeout, so large files can keep downloading while bytes
arrive. They do use an idle/no-progress timeout; if a stalled archive sends no bytes
or progress within that window, the download fails clearly.

Run `biomcp cache path` to print the managed HTTP cache directory on the current
machine without creating or migrating cache directories.

For freshness-sensitive workflows, use `--no-cache`.

## Authentication requirements

BioMCP only requires API keys for a subset of sources.

| Source | Environment variable | Required when |
|--------|----------------------|---------------|
| AlphaGenome | `ALPHAGENOME_API_KEY` | Running `get variant <id> predict` |
| Semantic Scholar | `S2_API_KEY` | Optional authenticated requests for `search article`, `get article`, `article batch`, TLDR, citation/reference/recommendation helpers, and `get article <id> fulltext --pdf` metadata enrichment |
| NCI CTS API | `NCI_API_KEY` | Trial operations with `--source nci` |
| OncoKB | `ONCOKB_TOKEN` | Running `variant oncokb <id>` |
| DisGeNET | `DISGENET_API_KEY` | Running `get gene <symbol> disgenet` or `get disease <name_or_id> disgenet` |
| NCBI E-utilities | `NCBI_API_KEY` | Optional; improves PubTator3, PubMed/efetch, PMC OA, and NCBI ID Converter quota headroom |
| OpenFDA | `OPENFDA_API_KEY` | Optional; improves quota headroom |
| UMLS | `UMLS_API_KEY` | Optional clinical crosswalk enrichment for `biomcp discover <query>` |

## Source-specific rate and payload constraints

Upstream services can change quotas without notice, so BioMCP documents enforced limits
and practical ceilings observed in command behavior.

| Source / command path | BioMCP-enforced limit | Practical guidance |
|-----------------------|-----------------------|--------------------|
| OpenFDA adverse-event / recall / device | `--limit` must be 1-50 | Use narrower filters and iterative queries for large pulls |
| CDC WONDER VAERS | Automated queries should run one at a time; CDC recommends about 2 minutes between repeated data-mining requests | Keep VAERS queries targeted, prefer fixture-frozen contract tests over live loops, and use `biomcp health --apis-only` for readiness checks |
| Gene search | `--limit` must be 1-50 | Start with small limits, then increase |
| Variant search | `--limit` must be 1-50 | Use `--gene` + `--consequence` to reduce noise |
| PGx (CPIC) | Rate-limited to 1 request / 250ms | Keep result limits focused around target gene/drug |
| PGx annotations (PharmGKB) | Rate-limited to 1 request / 500ms | Treat as enrichment; core PGx data remains from CPIC |
| GWAS search (`search gwas`) | `--limit` must be 1-50 | Prefer specific gene or trait queries to avoid broad result sets |
| Trial search | `--limit` defaults to 10, supports pagination | Use `--offset` to page and keep filters stable |
| Article search | `--limit` defaults to 10 | Use `--since` and typed entity filters to constrain results; `sort=relevance` defaults to hybrid for keyword queries and lexical for entity-only queries |
| KEGG pathway search/detail | Rate-limited to 1 request / 334ms | Matches KEGG's published 3 requests / second guidance |
| NIH Reporter funding sections | Rate-limited to 1 request / second | Use explicit gene symbols or disease phrases/identifiers; BioMCP queries the most recent 5 NIH fiscal years, keeps free-text disease lookups as-entered, falls back to the resolved canonical disease name for identifier lookups, and de-duplicates project-year rows before ranking grants |
| Semantic Scholar article helpers | 1 request / second with `S2_API_KEY`; 1 request / 2 seconds on the shared pool without it | Explicit helper commands fail fast on shared-pool `429` responses; set `S2_API_KEY` for dedicated quota and retry behavior |
| DisGeNET `disgenet` sections | Server-enforced; trial accounts may return first-page-only results and `429` with `X-Rate-Limit-Retry-After-Seconds` | Keep requests explicit, avoid fan-out loops, and retry after the server-provided cooldown |

## Trial source behavior

BioMCP supports two trial backends with similar command syntax but different retrieval behavior.

| Source flag | Backend | Strengths | Caveats |
|-------------|---------|-----------|---------|
| `--source ctgov` (default) | ClinicalTrials.gov API v2 | No API key, broad public coverage | Query behavior can vary with complex advanced terms |
| `--source nci` | NCI CTS API | Alternative indexing, oncology-focused source | Requires `NCI_API_KEY` and NCI-specific availability |

## Article pipeline behavior

Article workflows compose multiple APIs for different tasks:

1. PubTator3 + Europe PMC + PubMed for federated search, with LitSense2 added for keyword-bearing queries and an optional Semantic Scholar leg when the filter set is compatible (parallel fan-out, identifier-aware merge across PMID/PMCID/DOI, per-source capping after deduplication and before ranking, local lexical/semantic/hybrid relevance ranking)
2. Europe PMC for bibliographic metadata
3. PubTator3 for entity annotations
4. Semantic Scholar for the optional search leg, TLDR, citation graph, influential citation counts, recommendations, and `openAccessPdf` metadata for the explicit `--pdf` fallback
5. NCBI ID Converter bridges PMID or DOI to PMCID before PMCID-dependent full-text rungs when the base article lacks PMCID
6. Europe PMC PMC XML, NCBI EFetch PMC XML, PMC OA Archive XML, Europe PMC MED XML, PMC HTML, and opt-in Semantic Scholar PDF form the full-text content ladder where available

NCBI ID Converter bridges PMID or DOI to PMCID before PMCID-dependent full-text rungs.
Semantic Scholar supplies `openAccessPdf` metadata for the explicit `--pdf` fallback;
BioMCP fetches that third-party PDF URL only after the caller opts in.

This means metadata, annotations, and full text may have different availability
for the same PMID.

## OpenFDA behavior

OpenFDA drives three BioMCP features:

- FAERS drug adverse events
- Drug/device recalls
- MAUDE device events
- Diagnostic `regulatory` overlays from device 510(k) and PMA

OpenFDA may return no results for highly specific filters even when broader filters succeed.
Start broad (`--drug`, `--type`) and then tighten with `--reaction`, `--outcome`, `--classification`, or date filters.

## CDC WONDER VAERS behavior

CDC WONDER VAERS drives the aggregate vaccine branch of `search adverse-event`.

- `--source all` always keeps the OpenFDA FAERS path and adds VAERS only when the query resolves to a vaccine and the active filters are VAERS-compatible.
- `--source vaers` is aggregate-only and uses the CDC WONDER D8 XML POST contract rather than case-level VAERS report retrieval.
- CDC WONDER requires consent to its data use restrictions, and the public API guidance asks automated data-mining clients to send queries one at a time with recovery time between repeated requests.

## Provenance expectations

BioMCP output intentionally preserves source identity and record identifiers.
Users should always be able to trace:

- Which source produced the data
- Which identifier anchors the record (e.g., NCT, PMID, MONDO, rsID)
- Which sections come from direct source fields vs normalized rendering

## Operations checklist

When debugging source discrepancies:

1. Run `biomcp health --apis-only` to inspect upstream/API connectivity plus any excluded key-gated sources
2. Run `biomcp health` to inspect local readiness rows such as DDInter local data, EMA local data, WHO Prequalification local data, CDC CVX/MVX local data, GTR local data, WHO IVD local data, and cache dir
3. Treat `biomcp health` as an inspection surface: it does not currently exit non-zero on partial upstream failures
4. Run `./scripts/contract-smoke.sh --fast` for representative live probes, or `./scripts/contract-smoke.sh` for the fuller contract set
5. Retry with `--no-cache`
6. Confirm required API keys are set for optional sources
7. Switch source when applicable (`--source ctgov` vs `--source nci`)
8. Reduce filter complexity and retest

## Related docs

- [Quick Reference](quick-reference.md)
- [Error Codes](error-codes.md)
- [Troubleshooting](../troubleshooting.md)
