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
| Article search & metadata | PubTator3 + Europe PMC | `https://www.ncbi.nlm.nih.gov/research/pubtator3-api`, `https://www.ebi.ac.uk/europepmc/webservices/rest` | No | Federated search with PMID dedup and source-grouped output |
| Article enrichment and graph helpers | Semantic Scholar | `https://api.semanticscholar.org` | Optional (`S2_API_KEY`) | TLDR, influential citations, citation/reference graph, recommendations |
| Article annotations | PubTator3 | `https://www.ncbi.nlm.nih.gov/research/pubtator3-api` | No | Entity annotations |
| Article fulltext resolution | PMC OA + NCBI ID Converter | `https://www.ncbi.nlm.nih.gov/pmc/utils/oa/oa.fcgi`, `https://pmc.ncbi.nlm.nih.gov/tools/idconv/api/v1/articles` | No | Full-text and PMID/PMCID/DOI bridging |
| Drug | MyChem.info | `https://mychem.info/v1` | No | Drug metadata, targets, synonyms |
| Drug section enrichments | ChEMBL + OpenTargets | `https://www.ebi.ac.uk/chembl/api/data`, `https://api.platform.opentargets.org/api/v4/graphql` | No | Target and indication expansion sections |
| Disease normalization | MyDisease.info | `https://mydisease.info/v1` | No | MONDO-oriented disease normalization |
| Discover structured concepts | OLS4 | `https://www.ebi.ac.uk/ols4` | No | Free-text ontology search for `biomcp discover`; OLS4 is the required backbone |
| Discover clinical crosswalks | UMLS REST API | `https://uts-ws.nlm.nih.gov/rest` | Optional (`UMLS_API_KEY`) | Adds ICD-10, SNOMED CT, RxNorm, OMIM, and related cross-vocabulary IDs to discover results |
| Discover plain-language topics | MedlinePlus Search | `https://wsearch.nlm.nih.gov/ws/query` | No | Best-effort disease/symptom context for `biomcp discover`; suppressed for gene/drug/pathway flows |
| Phenotype term resolution | HPO JAX API | `https://ontology.jax.org/api/hp` | No | Direct HPO term lookup and normalization used by phenotype workflows |
| Disease genes/pathways/prevalence | OpenTargets GraphQL + Reactome | `https://api.platform.opentargets.org/api/v4/graphql`, `https://reactome.org/ContentService` | No | Baseline disease context with ranked associated targets and OpenTargets score summaries |
| Disease `genes` and `phenotypes` sections | Monarch Initiative API v3 | `https://api-v3.monarchinitiative.org` | No | Core disease associations and phenotype evidence |
| Disease `genes` and `variants` augmentation | CIViC | `https://civicdb.org/api` | No | Somatic driver augmentation for genes and disease-associated molecular profiles |
| Disease `models` section | Monarch Initiative API v3 | `https://api-v3.monarchinitiative.org` | No | Model-organism evidence with relationship and provenance |
| Disease `disgenet` section | DisGeNET REST API | `https://api.disgenet.com/api/v1` | Yes (`DISGENET_API_KEY`) | Ranked scored disease-gene associations; disease lookup uses UMLS-backed DisGeNET identifiers |
| Phenotype search (`search phenotype`) | Monarch Initiative API v3 | `https://api-v3.monarchinitiative.org` | No | HPO set similarity search to ranked diseases |
| PGx core interactions/recommendations | CPIC API | `https://api.cpicpgx.org/v1` | No | Pair, recommendation, frequency, and guideline views |
| PGx annotations section | PharmGKB API | `https://api.pharmgkb.org/v1` | No | Clinical/guideline/label annotation enrichment |
| Pathway | Reactome + KEGG + WikiPathways + g:Profiler | `https://reactome.org/ContentService`, `https://rest.kegg.jp`, `https://webservice.wikipathways.org`, `https://biit.cs.ut.ee/gprofiler/api` | No | Pathway search and detail use Reactome + KEGG + WikiPathways; `genes` are available across all three sources, while `events` and pathway `enrichment` remain Reactome-only; top-level `biomcp enrich` uses **g:Profiler** |
| Protein | UniProt + InterPro + STRING + ComplexPortal | `https://rest.uniprot.org`, `https://www.ebi.ac.uk/interpro/api`, `https://string-db.org/api`, `https://www.ebi.ac.uk/intact/complex-ws` | No | Protein cards, domains, interactions, structures, and human protein complex membership; structure IDs are surfaced from UniProt cross-references to PDB and AlphaFold DB |
| Drug/device safety, labels, shortages, and approvals | OpenFDA | `https://api.fda.gov` | Optional (`OPENFDA_API_KEY`) | FAERS, MAUDE, recalls, drug labels, shortages, and Drugs@FDA-derived approvals |
| Gene enrichment sections | Enrichr | `https://maayanlab.cloud/Enrichr` | No | Gene enrichment sections inside entity outputs use Enrichr; this is distinct from top-level `biomcp enrich` |
| Cohort frequencies (best effort) | cBioPortal | `https://www.cbioportal.org/api` | No | Supplemental cancer frequency context |

## Global HTTP behavior

All HTTP-based sources share a common client with:

- Connect timeout: 10 seconds
- Request timeout: 30 seconds
- Retries: exponential backoff, up to 3 retries for transient failures
- Disk cache: `~/.cache/biomcp/http-cacache` (platform-adjusted cache root)

For freshness-sensitive workflows, use `--no-cache`.

## Authentication requirements

BioMCP only requires API keys for a subset of sources.

| Source | Environment variable | Required when |
|--------|----------------------|---------------|
| AlphaGenome | `ALPHAGENOME_API_KEY` | Running `get variant <id> predict` |
| Semantic Scholar | `S2_API_KEY` | Running `article citations|references|recommendations`; enriching `get article` with TLDR and influence data |
| NCI CTS API | `NCI_API_KEY` | Trial operations with `--source nci` |
| OncoKB | `ONCOKB_TOKEN` | Running `variant oncokb <id>` |
| DisGeNET | `DISGENET_API_KEY` | Running `get gene <symbol> disgenet` or `get disease <name_or_id> disgenet` |
| NCBI E-utilities | `NCBI_API_KEY` | Optional; improves PubTator3, PMC OA, and NCBI ID Converter quota headroom |
| OpenFDA | `OPENFDA_API_KEY` | Optional; improves quota headroom |
| UMLS | `UMLS_API_KEY` | Optional clinical crosswalk enrichment for `biomcp discover <query>` |

## Source-specific rate and payload constraints

Upstream services can change quotas without notice, so BioMCP documents enforced limits
and practical ceilings observed in command behavior.

| Source / command path | BioMCP-enforced limit | Practical guidance |
|-----------------------|-----------------------|--------------------|
| OpenFDA adverse-event / recall / device | `--limit` must be 1-50 | Use narrower filters and iterative queries for large pulls |
| Gene search | `--limit` must be 1-50 | Start with small limits, then increase |
| Variant search | `--limit` must be 1-50 | Use `--gene` + `--consequence` to reduce noise |
| PGx (CPIC) | Rate-limited to 1 request / 250ms | Keep result limits focused around target gene/drug |
| PGx annotations (PharmGKB) | Rate-limited to 1 request / 500ms | Treat as enrichment; core PGx data remains from CPIC |
| GWAS search (`search gwas`) | `--limit` must be 1-50 | Prefer specific gene or trait queries to avoid broad result sets |
| Trial search | `--limit` defaults to 10, supports pagination | Use `--offset` to page and keep filters stable |
| Article search | `--limit` defaults to 10 | Use `--since` and entity filters to constrain results |
| KEGG pathway search/detail | Rate-limited to 1 request / 334ms | Matches KEGG's published 3 requests / second guidance |
| Semantic Scholar article helpers | 1 request / second, process-local | Use explicit helper commands and batch normalization for multi-paper recommendation inputs |
| DisGeNET `disgenet` sections | Server-enforced; trial accounts may return first-page-only results and `429` with `X-Rate-Limit-Retry-After-Seconds` | Keep requests explicit, avoid fan-out loops, and retry after the server-provided cooldown |

## Trial source behavior

BioMCP supports two trial backends with similar command syntax but different retrieval behavior.

| Source flag | Backend | Strengths | Caveats |
|-------------|---------|-----------|---------|
| `--source ctgov` (default) | ClinicalTrials.gov API v2 | No API key, broad public coverage | Query behavior can vary with complex advanced terms |
| `--source nci` | NCI CTS API | Alternative indexing, oncology-focused source | Requires `NCI_API_KEY` and NCI-specific availability |

## Article pipeline behavior

Article workflows compose multiple APIs for different tasks:

1. PubTator3 + Europe PMC for federated search (parallel fan-out, PMID dedup, PubTator-priority merge for duplicate PMIDs)
2. Europe PMC for bibliographic metadata
3. PubTator3 for entity annotations
4. Semantic Scholar for optional TLDR, citation graph, influential citation counts, and recommendations
5. NCBI ID converter + PMC OA for full-text resolution where available

This means metadata, annotations, and fulltext may have different availability for the same PMID.

## OpenFDA behavior

OpenFDA drives three BioMCP features:

- FAERS drug adverse events
- Drug/device recalls
- MAUDE device events

OpenFDA may return no results for highly specific filters even when broader filters succeed.
Start broad (`--drug`, `--type`) and then tighten with `--reaction`, `--outcome`, `--classification`, or date filters.

## Provenance expectations

BioMCP output intentionally preserves source identity and record identifiers.
Users should always be able to trace:

- Which source produced the data
- Which identifier anchors the record (e.g., NCT, PMID, MONDO, rsID)
- Which sections come from direct source fields vs normalized rendering

## Operations checklist

When debugging source discrepancies:

1. Run `biomcp health --apis-only` to inspect per-source connectivity plus any excluded key-gated sources
2. Treat `biomcp health` as an inspection surface: it does not currently exit non-zero on partial upstream failures
3. Run `./scripts/contract-smoke.sh --fast` for representative live probes, or `./scripts/contract-smoke.sh` for the fuller contract set
4. Retry with `--no-cache`
5. Confirm required API keys are set for optional sources
6. Switch source when applicable (`--source ctgov` vs `--source nci`)
7. Reduce filter complexity and retest

## Related docs

- [Quick Reference](quick-reference.md)
- [Error Codes](error-codes.md)
- [Troubleshooting](../troubleshooting.md)
