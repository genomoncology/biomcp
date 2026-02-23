# Data Sources

BioMCP unifies multiple biomedical data providers behind one CLI grammar.
This reference explains source provenance, authentication requirements, base endpoints,
and operational caveats so users can reason about result quality and troubleshooting.

## Source matrix

| Entity / feature | Primary source(s) | Base URL | Auth required | Notes |
|------------------|-------------------|----------|---------------|-------|
| Gene | MyGene.info | `https://mygene.info/v3` | No | Symbol lookup, aliases, summaries |
| Gene sections | UniProt, QuickGO, STRING | `https://rest.uniprot.org`, `https://www.ebi.ac.uk/QuickGO/services`, `https://string-db.org/api` | No | Protein summary, GO terms, interaction network |
| Variant | MyVariant.info | `https://myvariant.info/v1` | No | rsID/HGVS lookup, ClinVar and population annotations |
| Variant population section | MyVariant.info (gnomAD fields) | `https://myvariant.info/v1` | No | Uses cached gnomAD AF/subpopulation fields from MyVariant payload |
| Variant GWAS section and GWAS search | GWAS Catalog REST API | `https://www.ebi.ac.uk/gwas/rest/api` | No | rsID, gene, and trait association retrieval |
| Variant OncoKB helper | OncoKB | `https://www.oncokb.org/api/v1` | Yes (`ONCOKB_TOKEN`) | Accessed via explicit `variant oncokb <id>` command |
| Variant prediction | AlphaGenome | `https://gdmscience.googleapis.com:443` | Yes (`ALPHAGENOME_API_KEY`) | gRPC scoring for `predict` section |
| Trial (default) | ClinicalTrials.gov API v2 | `https://clinicaltrials.gov/api/v2` | No | Default trial search/get source |
| Trial (optional) | NCI CTS API | `https://clinicaltrialsapi.cancer.gov/api/v2` | Yes (`NCI_API_KEY`) | Enabled via `--source nci` |
| NCI vocabulary search | NCI CTS API | `https://clinicaltrialsapi.cancer.gov/api/v2` | Yes (`NCI_API_KEY`) | `search organization`, `search intervention`, `search biomarker` |
| Article metadata | Europe PMC + PubMed | `https://www.ebi.ac.uk/europepmc/webservices/rest` | No | Search and bibliographic metadata |
| Article annotations | PubTator3 | `https://www.ncbi.nlm.nih.gov/research/pubtator3-api` | No | Entity annotations |
| Article fulltext resolution | PMC OA + NCBI ID Converter | `https://www.ncbi.nlm.nih.gov/pmc/utils/oa/oa.fcgi`, `https://pmc.ncbi.nlm.nih.gov/tools/idconv/api/v1/articles` | No | Full-text and PMID/PMCID/DOI bridging |
| Drug | MyChem.info | `https://mychem.info/v1` | No | Drug metadata, targets, synonyms |
| Drug section enrichments | ChEMBL + OpenTargets | `https://www.ebi.ac.uk/chembl/api/data`, `https://api.platform.opentargets.org/api/v4/graphql` | No | Target and indication expansion sections |
| Disease normalization | MyDisease.info | `https://mydisease.info/v1` | No | MONDO-oriented disease normalization |
| Disease genes/pathways/prevalence | OpenTargets GraphQL + Reactome | `https://api.platform.opentargets.org/api/v4/graphql`, `https://reactome.org/ContentService` | No | Baseline disease context |
| Disease `genes` and `phenotypes` sections | Monarch Initiative API v3 | `https://api-v3.monarchinitiative.org` | No | Core disease associations and phenotype evidence |
| Disease `genes` and `variants` augmentation | CIViC | `https://civicdb.org/api` | No | Somatic driver augmentation for genes and disease-associated molecular profiles |
| Disease `models` section | Monarch Initiative API v3 | `https://api-v3.monarchinitiative.org` | No | Model-organism evidence with relationship and provenance |
| Phenotype search (`search phenotype`) | Monarch Initiative API v3 | `https://api-v3.monarchinitiative.org` | No | HPO set similarity search to ranked diseases |
| PGx core interactions/recommendations | CPIC API | `https://api.cpicpgx.org/v1` | No | Pair, recommendation, frequency, and guideline views |
| PGx annotations section | PharmGKB API | `https://api.pharmgkb.org/v1` | No | Clinical/guideline/label annotation enrichment |
| Pathway | Reactome + g:Profiler | `https://reactome.org/ContentService`, `https://biit.cs.ut.ee/gprofiler/api` | No | Pathway search, events, participants, enrichment |
| Protein | UniProt + InterPro + STRING | `https://rest.uniprot.org`, `https://www.ebi.ac.uk/interpro/api`, `https://string-db.org/api` | No | Protein cards, domains, interactions, structures |
| Adverse events and recalls | OpenFDA | `https://api.fda.gov` | Optional (`OPENFDA_API_KEY`) | FAERS, recalls, and MAUDE device events |
| Gene enrichment sections | Enrichr | `https://maayanlab.cloud/Enrichr` | No | Ontology/disease enrichment sections |
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
| NCI CTS API | `NCI_API_KEY` | Trial operations with `--source nci`, or `search organization/intervention/biomarker` |
| OncoKB | `ONCOKB_TOKEN` | Running `variant oncokb <id>` |
| OpenFDA | `OPENFDA_API_KEY` | Optional; improves quota headroom |

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

## Trial source behavior

BioMCP supports two trial backends with similar command syntax but different retrieval behavior.

| Source flag | Backend | Strengths | Caveats |
|-------------|---------|-----------|---------|
| `--source ctgov` (default) | ClinicalTrials.gov API v2 | No API key, broad public coverage | Query behavior can vary with complex advanced terms |
| `--source nci` | NCI CTS API | Alternative indexing, oncology-focused source | Requires `NCI_API_KEY` and NCI-specific availability |

## Article pipeline behavior

Article workflows compose multiple APIs for different tasks:

1. Europe PMC / PubMed for search and bibliographic metadata
2. PubTator3 for annotations
3. NCBI ID converter + PMC OA for full-text resolution where available

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

1. Run `biomcp health --apis-only`
2. Retry with `--no-cache`
3. Confirm required API keys are set for optional sources
4. Switch source when applicable (`--source ctgov` vs `--source nci`)
5. Reduce filter complexity and retest

## Related docs

- [Quick Reference](quick-reference.md)
- [Error Codes](error-codes.md)
- [Troubleshooting](../troubleshooting.md)
