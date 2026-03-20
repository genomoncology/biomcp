# Source Licensing and Terms

BioMCP itself is MIT-licensed, but the data providers it queries are not all licensed the same way.

Three distinctions matter:

- BioMCP does not vendor, mirror, or ship upstream datasets in the repository.
- BioMCP performs on-demand read-only queries against upstream services.
- Returned records, downloaded full text, saved output, and downstream reuse can still be governed by upstream provider terms.

Use this page for provider terms, licensing, redistribution, and account requirements. Use [API Keys](../getting-started/api-keys.md) for setup steps and [Data Sources](data-sources.md) for runtime behavior, endpoints, rate limits, and operational caveats.

The canonical machine-readable inventory for this page lives in [`sources.json`](sources.json).

## How to read this page

- `tier`
  Tier `1` means baseline BioMCP usage works without credentials.
  Tier `2` means the BioMCP feature needs provider credentials, registration, or a provider-controlled account path.
  Tier `3` means BioMCP can query the provider, but notable reuse, redistribution, attribution, or policy caveats need explicit attention.
- `integration_mode`
  `direct_api` means BioMCP calls the provider directly.
  `indirect_only` means BioMCP only surfaces that provider as provenance through another API payload.
- `bioMcp_auth`
  `none` means BioMCP does not need a key for that provider.
  `optional_env` means an environment variable improves quota or access quality but is not required for baseline use.
  `required_env` means the feature requires a configured environment variable.
  `not_applicable` means there is no standalone BioMCP auth path because the provider is indirect-only.

## Summary table

| Source | Tier | Mode | BioMCP auth | License / terms summary | Redistribution summary | Terms URL |
|---|---|---|---|---|---|---|
| AlphaGenome | 2 | direct_api | required_env | custom provider terms; access is gated by Google/DeepMind service controls | do not assume open redistribution rights for returned prediction outputs | <https://deepmind.google/science/alphagenome/> |
| cBioPortal | 3 | direct_api | none | public API with study-specific downstream terms | reuse depends on the specific study or consortium behind each dataset | <https://www.cbioportal.org/> |
| ChEMBL | 1 | direct_api | none | EMBL-EBI open data service; ChEMBL is published for broad reuse | reuse is generally allowed under the provider's open-data terms with attribution where required | <https://www.ebi.ac.uk/chembl/> |
| CIViC | 1 | direct_api | none | open community knowledgebase; CIViC content is published for unrestricted reuse | reuse is broadly permitted; attribution remains best practice | <https://civicdb.org/home> |
| ClinGen | 1 | direct_api | none | public ClinGen curation resources with publication and attribution expectations | generally queryable and reusable, but users should preserve attribution and source context | <https://clinicalgenome.org/> |
| ClinicalTrials.gov | 1 | direct_api | none | U.S. government public information service | records are broadly reusable; preserve identifiers and avoid implying NLM endorsement | <https://clinicaltrials.gov/data-api/about-api> |
| ComplexPortal | 1 | direct_api | none | EMBL-EBI open data service | reuse follows EMBL-EBI resource terms and any embedded third-party source obligations | <https://www.ebi.ac.uk/complexportal/> |
| CPIC | 1 | direct_api | none | CPIC content is published under CC0 with trademark and attribution guidance | content reuse is broadly allowed, but the CPIC mark/logo has separate restrictions | <https://cpicpgx.org/license/> |
| DGIdb | 1 | direct_api | none | open interaction service; aggregated claims may still reflect upstream source terms | treat DGIdb as an aggregation layer and preserve source attribution for underlying claim providers | <https://www.dgidb.org/about> |
| DisGeNET | 2 | direct_api | required_env | custom provider terms for API and downloads | do not assume unrestricted redistribution; use according to the provider account terms | <https://www.disgenet.com/> |
| Enrichr | 1 | direct_api | none | open web/API service with citation expectations for Enrichr and its libraries | reuse of results should preserve attribution to Enrichr and the underlying enrichment libraries | <https://maayanlab.cloud/Enrichr/> |
| Europe PMC | 1 | direct_api | none | open literature metadata service; article and full-text licenses vary by record | metadata is broadly reusable, but full text and PDFs remain governed by article-level licenses | <https://europepmc.org/RestfulWebService> |
| gnomAD | 3 | direct_api | none | Broad Institute data policies with attribution and service-specific conditions | querying is open, but users should review the gnomAD policies before bulk reuse or republishing | <https://gnomad.broadinstitute.org/policies> |
| g:Profiler | 1 | direct_api | none | open enrichment service with provider citation expectations | results are queryable and reusable, but cite g:Profiler and any underlying databases you depend on | <https://biit.cs.ut.ee/gprofiler/help.cgi> |
| GTEx | 1 | direct_api | none | NIH-hosted public-access expression resource | public summary/expression views are broadly reusable; controlled-access data remains outside BioMCP's scope | <https://gtexportal.org/home/documentationPage> |
| GWAS Catalog | 1 | direct_api | none | EMBL-EBI resource terms; summary statistics may carry separate licenses | query results are generally reusable, but dataset-level summary statistics can have separate downstream terms | <https://www.ebi.ac.uk/gwas/docs/about> |
| Human Protein Atlas | 3 | direct_api | none | CC BY-SA 4.0 for copyrightable parts of the database | reuse is allowed with attribution and ShareAlike; third-party components may impose extra conditions | <https://www.proteinatlas.org/about/licence> |
| HPO JAX API | 1 | direct_api | none | open HPO data with attribution and integrity requirements | reuse is allowed, but users should preserve attribution, version context, and source integrity | <https://human-phenotype-ontology.github.io/license.html> |
| InterPro | 1 | direct_api | none | EMBL-EBI open data resource | reuse follows InterPro/EMBL-EBI resource terms and any embedded member-database obligations | <https://www.ebi.ac.uk/interpro/> |
| KEGG | 3 | direct_api | none | custom KEGG terms; academic users may freely use the website, non-academic use requires a commercial license | do not assume commercial redistribution rights; query access does not grant a redistribution license | <https://www.kegg.jp/kegg/legal.html> |
| MedlinePlus | 1 | direct_api | none | NLM public-information service with trademark and endorsement guidance | content is widely reusable, but preserve attribution and avoid implying MedlinePlus/NLM endorsement | <https://medlineplus.gov/about/using/> |
| Monarch Initiative | 1 | direct_api | none | open integrated knowledge graph; underlying source licenses still matter | results can be queried openly, but downstream reuse should respect the original sources folded into Monarch | <https://monarchinitiative.org/> |
| MyChem.info | 1 | direct_api | none | BioThings aggregation service; upstream source terms continue to apply | do not assume aggregator responses are relicensed; preserve source provenance for downstream reuse | <https://docs.mychem.info/en/latest/> |
| MyDisease.info | 1 | direct_api | none | BioThings aggregation service; source ontologies and datasets keep their own terms | treat payloads as aggregated source data rather than a new umbrella license | <https://docs.mydisease.info/en/latest/> |
| MyGene.info | 1 | direct_api | none | BioThings aggregation service; source-specific terms remain attached to underlying records | reuse should preserve provenance back to NCBI Gene, UniProt, and other upstream sources | <https://docs.mygene.info/en/latest/> |
| MyVariant.info | 1 | direct_api | none | BioThings aggregation service; indirect providers retain their own terms | ClinVar, COSMIC, Cancer Genome Interpreter, and gnomAD-related fields should be treated according to their original providers' terms | <https://docs.myvariant.info/en/latest/> |
| NCBI ID Converter | 1 | direct_api | optional_env | NLM public-domain utility service | utility results are broadly reusable; keep article-level identifiers and downstream article licenses distinct | <https://pmc.ncbi.nlm.nih.gov/tools/idconv/> |
| NCI CTS | 2 | direct_api | required_env | custom provider API terms for the NCI Clinical Trials Search API | query output is usable for search and review, but downstream reuse should follow NCI API terms and record provenance | <https://clinicaltrialsapi.cancer.gov/> |
| OLS4 | 1 | direct_api | none | EMBL-EBI ontology browser; each ontology keeps its own license | ontology metadata is queryable, but downstream reuse depends on the specific ontology surfaced | <https://www.ebi.ac.uk/ols4/> |
| OncoKB | 2 | direct_api | required_env | custom provider terms; academic research access is no-fee but licensed, commercial/clinical use requires a paid license | do not assume open redistribution rights for OncoKB data or proprietary treatment descriptions | <https://faq.oncokb.org/licensing> |
| OpenFDA | 1 | direct_api | optional_env | FDA-origin public data and API terms | data is broadly reusable, but avoid implying FDA endorsement and preserve source context | <https://open.fda.gov/apis/authentication/> |
| OpenTargets | 1 | direct_api | none | Open Targets data is CC0; platform code is Apache 2.0 | platform data is dedicated to the public domain, but linked evidence still carries source provenance | <https://platform-docs.opentargets.org/licence> |
| PharmGKB | 3 | direct_api | none | ClinPGx API data is CC BY-SA 4.0 and subject to the provider's data usage policy | reuse is allowed with attribution and ShareAlike; some underlying annotations and external assets may add extra constraints | <https://api.pharmgkb.org/> |
| PMC OA | 1 | direct_api | optional_env | open-access subset only; article licenses vary within PMC OA | full text is reusable only according to each article's specific PMC Open Access license | <https://pmc.ncbi.nlm.nih.gov/tools/openftlist/> |
| PubTator3 | 1 | direct_api | optional_env | NCBI/NLM public-domain annotation service | results are broadly reusable, but preserve PMID/source provenance and article-level rights separately | <https://www.ncbi.nlm.nih.gov/research/pubtator3/api> |
| QuickGO | 1 | direct_api | none | GO/EMBL-EBI open data service | query results are generally reusable; preserve GO/EMBL-EBI attribution where expected | <https://www.ebi.ac.uk/QuickGO/> |
| Reactome | 1 | direct_api | none | Reactome pathway content is CC BY 4.0, with some data exports additionally placed under CC0 | reuse is allowed with attribution; preserve pathway/source provenance in downstream materials | <https://reactome.org/license> |
| Semantic Scholar | 2 | direct_api | required_env | custom API license agreement | the API license restricts repackaging, resale, and broad commercial redistribution without expanded licensing | <https://www.semanticscholar.org/product/api/license> |
| STRING | 1 | direct_api | none | CC BY 4.0 | reuse is allowed with attribution to STRING and the original publication/resource | <https://string-db.org/cgi/access?footer_active_subpage=licensing> |
| UMLS | 2 | direct_api | required_env | custom UMLS Metathesaurus license and terminology-specific appendices | do not assume unrestricted redistribution; some embedded vocabularies add their own restrictions or affiliate licenses | <https://www.nlm.nih.gov/databases/umls.html> |
| UniProt | 1 | direct_api | none | CC BY 4.0 | reuse is allowed with attribution; linked cross-references can have their own terms | <https://www.uniprot.org/help/license> |
| WikiPathways | 1 | direct_api | none | CC0 | pathway content is dedicated to the public domain; attribution is still good scholarly practice | <https://classic.wikipathways.org/index.php/WikiPathways:License_Terms> |
| AlphaFold DB | 1 | indirect_only | not_applicable | AlphaFold DB structural predictions are published for broad open use | reuse is generally open, but preserve model/source provenance and article citations | <https://alphafold.ebi.ac.uk/faq> |
| Cancer Genome Interpreter | 3 | indirect_only | not_applicable | custom tool terms | do not assume commercial reuse rights; the official terms restrict some external and commercial use | <https://www.cancergenomeinterpreter.org/conditions> |
| ClinVar | 1 | indirect_only | not_applicable | NCBI public-domain submission archive | records are broadly reusable, but preserve accession/provenance and submitter context | <https://www.ncbi.nlm.nih.gov/clinvar/docs/maintenance_use/> |
| COSMIC | 3 | indirect_only | not_applicable | custom COSMIC licensing with commercial restrictions | direct redistribution and direct integration remain intentionally unsupported without a separate COSMIC license | <https://www.sanger.ac.uk/legal/cosmic-licensing/> |
| Disease Ontology | 1 | indirect_only | not_applicable | open disease ontology project | reuse is generally open; preserve ontology version and source references | <https://disease-ontology.org/> |
| DrugBank | 3 | indirect_only | not_applicable | custom DrugBank terms of use and licensing | use or redistribution of DrugBank content requires a DrugBank license; do not assume open downstream rights | <https://trust.drugbank.com/drugbank-trust-center/drugbank-terms-of-service> |
| Drugs@FDA | 1 | indirect_only | not_applicable | FDA-origin public information | approval records are broadly reusable; avoid implying FDA endorsement | <https://open.fda.gov/apis/drug/drugsfda/> |
| MONDO | 1 | indirect_only | not_applicable | CC BY 4.0 | reuse is allowed with attribution and ontology version tracking | <https://mondo.monarchinitiative.org/pages/download/> |
| PDB | 1 | indirect_only | not_applicable | PDB archive data is CC0 1.0 | data is broadly reusable; attribution to original structure authors is encouraged | <https://www.rcsb.org/pages/usage-policy> |

## Tier 1 - Baseline use without credentials

### ChEMBL

- BioMCP surfaces: `get drug <name> targets; get drug <name> indications`
- Integration mode: `direct_api`
- BioMCP auth: `none`
- Provider access / registration: open public API
- License / terms summary: EMBL-EBI open data service; ChEMBL is published for broad reuse
- Redistribution / reuse summary: reuse is generally allowed under the provider's open-data terms with attribution where required
- Official terms URL: <https://www.ebi.ac.uk/chembl/>
- Reviewed on: `2026-03-20`
- Notes: BioMCP only queries live ChEMBL endpoints and does not ship ChEMBL data in the repository.

### CIViC

- BioMCP surfaces: `get gene <symbol> civic; get disease <id> variants`
- Integration mode: `direct_api`
- BioMCP auth: `none`
- Provider access / registration: open public API
- License / terms summary: open community knowledgebase; CIViC content is published for unrestricted reuse
- Redistribution / reuse summary: reuse is broadly permitted; attribution remains best practice
- Official terms URL: <https://civicdb.org/home>
- Reviewed on: `2026-03-20`
- Notes: CIViC is treated here as an open-access evidence source surfaced directly by BioMCP.

### ClinGen

- BioMCP surfaces: `get gene <symbol> clingen`
- Integration mode: `direct_api`
- BioMCP auth: `none`
- Provider access / registration: public web/API access
- License / terms summary: public ClinGen curation resources with publication and attribution expectations
- Redistribution / reuse summary: generally queryable and reusable, but users should preserve attribution and source context
- Official terms URL: <https://clinicalgenome.org/>
- Reviewed on: `2026-03-20`
- Notes: Open Targets currently lists ClinGen under CC0 for its own ingestion, but BioMCP links to ClinGen's official project site because that is the provider surface users encounter directly.

### ClinicalTrials.gov

- BioMCP surfaces: `search trial; get trial <nct_id>`
- Integration mode: `direct_api`
- BioMCP auth: `none`
- Provider access / registration: open public API
- License / terms summary: U.S. government public information service
- Redistribution / reuse summary: records are broadly reusable; preserve identifiers and avoid implying NLM endorsement
- Official terms URL: <https://clinicaltrials.gov/data-api/about-api>
- Reviewed on: `2026-03-20`
- Notes: BioMCP uses the public v2 API as the baseline trial backend.

### ComplexPortal

- BioMCP surfaces: `get protein <id> complexes`
- Integration mode: `direct_api`
- BioMCP auth: `none`
- Provider access / registration: open public API
- License / terms summary: EMBL-EBI open data service
- Redistribution / reuse summary: reuse follows EMBL-EBI resource terms and any embedded third-party source obligations
- Official terms URL: <https://www.ebi.ac.uk/complexportal/>
- Reviewed on: `2026-03-20`
- Notes: BioMCP calls the IntAct Complex Portal web service. Complex membership data is queried on demand only.

### CPIC

- BioMCP surfaces: `search pgx; get pgx <gene_or_drug>`
- Integration mode: `direct_api`
- BioMCP auth: `none`
- Provider access / registration: open public API
- License / terms summary: CPIC content is published under CC0 with trademark and attribution guidance
- Redistribution / reuse summary: content reuse is broadly allowed, but the CPIC mark/logo has separate restrictions
- Official terms URL: <https://cpicpgx.org/license/>
- Reviewed on: `2026-03-20`
- Notes: CPIC announced in March 2026 that content is moving to ClinPGx, but current CPIC URLs continue to resolve.

### DGIdb

- BioMCP surfaces: `get gene <symbol> interactions; get drug <name> interactions`
- Integration mode: `direct_api`
- BioMCP auth: `none`
- Provider access / registration: open public API
- License / terms summary: open interaction service; aggregated claims may still reflect upstream source terms
- Redistribution / reuse summary: treat DGIdb as an aggregation layer and preserve source attribution for underlying claim providers
- Official terms URL: <https://www.dgidb.org/about>
- Reviewed on: `2026-03-20`
- Notes: DGIdb itself is open to query, but it aggregates claims from many external drug-gene sources.

### Enrichr

- BioMCP surfaces: `get gene <symbol> ontology`
- Integration mode: `direct_api`
- BioMCP auth: `none`
- Provider access / registration: open public API
- License / terms summary: open web/API service with citation expectations for Enrichr and its libraries
- Redistribution / reuse summary: reuse of results should preserve attribution to Enrichr and the underlying enrichment libraries
- Official terms URL: <https://maayanlab.cloud/Enrichr/>
- Reviewed on: `2026-03-20`
- Notes: Gene enrichment sections inside BioMCP use Enrichr; top-level `biomcp enrich` uses g:Profiler instead.

### Europe PMC

- BioMCP surfaces: `search article; get article <pmid>`
- Integration mode: `direct_api`
- BioMCP auth: `none`
- Provider access / registration: open public API
- License / terms summary: open literature metadata service; article and full-text licenses vary by record
- Redistribution / reuse summary: metadata is broadly reusable, but full text and PDFs remain governed by article-level licenses
- Official terms URL: <https://europepmc.org/RestfulWebService>
- Reviewed on: `2026-03-20`
- Notes: BioMCP uses Europe PMC for search and bibliographic metadata. Open-access reuse depends on the publication license attached to each record.

### g:Profiler

- BioMCP surfaces: `enrich <GENE1,GENE2,...>`
- Integration mode: `direct_api`
- BioMCP auth: `none`
- Provider access / registration: open public API
- License / terms summary: open enrichment service with provider citation expectations
- Redistribution / reuse summary: results are queryable and reusable, but cite g:Profiler and any underlying databases you depend on
- Official terms URL: <https://biit.cs.ut.ee/gprofiler/help.cgi>
- Reviewed on: `2026-03-20`
- Notes: BioMCP uses g:Profiler only for top-level gene-set enrichment.

### GTEx

- BioMCP surfaces: `get gene <symbol> expression`
- Integration mode: `direct_api`
- BioMCP auth: `none`
- Provider access / registration: open public portal/API for public-access expression data
- License / terms summary: NIH-hosted public-access expression resource
- Redistribution / reuse summary: public summary/expression views are broadly reusable; controlled-access data remains outside BioMCP's scope
- Official terms URL: <https://gtexportal.org/home/documentationPage>
- Reviewed on: `2026-03-20`
- Notes: BioMCP only queries public GTEx expression endpoints, not controlled-access donor-level data.

### GWAS Catalog

- BioMCP surfaces: `search gwas; get variant <id> gwas`
- Integration mode: `direct_api`
- BioMCP auth: `none`
- Provider access / registration: open public API
- License / terms summary: EMBL-EBI resource terms; summary statistics may carry separate licenses
- Redistribution / reuse summary: query results are generally reusable, but dataset-level summary statistics can have separate downstream terms
- Official terms URL: <https://www.ebi.ac.uk/gwas/docs/about>
- Reviewed on: `2026-03-20`
- Notes: Open Targets reports GWAS Catalog summary statistics under CC0 while the service remains under EMBL-EBI terms.

### HPO JAX API

- BioMCP surfaces: `search phenotype; discover phenotype term resolution`
- Integration mode: `direct_api`
- BioMCP auth: `none`
- Provider access / registration: open public ontology API
- License / terms summary: open HPO data with attribution and integrity requirements
- Redistribution / reuse summary: reuse is allowed, but users should preserve attribution, version context, and source integrity
- Official terms URL: <https://human-phenotype-ontology.github.io/license.html>
- Reviewed on: `2026-03-20`
- Notes: The HPO project's own license page is more specific than generic JAX site text and is the clearest official usage statement currently exposed.

### InterPro

- BioMCP surfaces: `get protein <id> domains`
- Integration mode: `direct_api`
- BioMCP auth: `none`
- Provider access / registration: open public API
- License / terms summary: EMBL-EBI open data resource
- Redistribution / reuse summary: reuse follows InterPro/EMBL-EBI resource terms and any embedded member-database obligations
- Official terms URL: <https://www.ebi.ac.uk/interpro/>
- Reviewed on: `2026-03-20`
- Notes: InterPro aggregates signatures from multiple member databases; downstream interpretation should keep that provenance.

### MedlinePlus

- BioMCP surfaces: `discover plain-language topics`
- Integration mode: `direct_api`
- BioMCP auth: `none`
- Provider access / registration: open public search API
- License / terms summary: NLM public-information service with trademark and endorsement guidance
- Redistribution / reuse summary: content is widely reusable, but preserve attribution and avoid implying MedlinePlus/NLM endorsement
- Official terms URL: <https://medlineplus.gov/about/using/>
- Reviewed on: `2026-03-20`
- Notes: BioMCP uses MedlinePlus only for best-effort plain-language discover context.

### Monarch Initiative

- BioMCP surfaces: `get disease <id> genes; get disease <id> models; search phenotype`
- Integration mode: `direct_api`
- BioMCP auth: `none`
- Provider access / registration: open public API
- License / terms summary: open integrated knowledge graph; underlying source licenses still matter
- Redistribution / reuse summary: results can be queried openly, but downstream reuse should respect the original sources folded into Monarch
- Official terms URL: <https://monarchinitiative.org/>
- Reviewed on: `2026-03-20`
- Notes: BioMCP uses Monarch as an aggregator for disease, phenotype, and model-organism relationships rather than as the legal source of every embedded assertion.

### MyChem.info

- BioMCP surfaces: `search drug; get drug <name>`
- Integration mode: `direct_api`
- BioMCP auth: `none`
- Provider access / registration: open public API
- License / terms summary: BioThings aggregation service; upstream source terms continue to apply
- Redistribution / reuse summary: do not assume aggregator responses are relicensed; preserve source provenance for downstream reuse
- Official terms URL: <https://docs.mychem.info/en/latest/>
- Reviewed on: `2026-03-20`
- Notes: DrugBank and other upstream providers appear in MyChem payloads with their own terms.

### MyDisease.info

- BioMCP surfaces: `search disease; get disease <id>`
- Integration mode: `direct_api`
- BioMCP auth: `none`
- Provider access / registration: open public API
- License / terms summary: BioThings aggregation service; source ontologies and datasets keep their own terms
- Redistribution / reuse summary: treat payloads as aggregated source data rather than a new umbrella license
- Official terms URL: <https://docs.mydisease.info/en/latest/>
- Reviewed on: `2026-03-20`
- Notes: Disease Ontology and MONDO appear through MyDisease.info as indirect provenance sources.

### MyGene.info

- BioMCP surfaces: `search gene; get gene <symbol>`
- Integration mode: `direct_api`
- BioMCP auth: `none`
- Provider access / registration: open public API
- License / terms summary: BioThings aggregation service; source-specific terms remain attached to underlying records
- Redistribution / reuse summary: reuse should preserve provenance back to NCBI Gene, UniProt, and other upstream sources
- Official terms URL: <https://docs.mygene.info/en/latest/>
- Reviewed on: `2026-03-20`
- Notes: BioMCP uses MyGene.info mainly as an identity/normalization layer rather than as the legal origin of all gene data.

### MyVariant.info

- BioMCP surfaces: `search variant; get variant <id>`
- Integration mode: `direct_api`
- BioMCP auth: `none`
- Provider access / registration: open public API
- License / terms summary: BioThings aggregation service; indirect providers retain their own terms
- Redistribution / reuse summary: ClinVar, COSMIC, Cancer Genome Interpreter, and gnomAD-related fields should be treated according to their original providers' terms
- Official terms URL: <https://docs.myvariant.info/en/latest/>
- Reviewed on: `2026-03-20`
- Notes: BioMCP intentionally documents several indirect-only provenance rows that arrive through MyVariant.info payloads.

### NCBI ID Converter

- BioMCP surfaces: `get article <id> fulltext`
- Integration mode: `direct_api`
- BioMCP auth: `optional_env` via `NCBI_API_KEY`
- Provider access / registration: open public utility; optional My NCBI API key improves throughput
- License / terms summary: NLM public-domain utility service
- Redistribution / reuse summary: utility results are broadly reusable; keep article-level identifiers and downstream article licenses distinct
- Official terms URL: <https://pmc.ncbi.nlm.nih.gov/tools/idconv/>
- API key / account URL: <https://www.ncbi.nlm.nih.gov/account/settings/>
- Reviewed on: `2026-03-20`
- Notes: NCBI API keys increase throughput but are not required for baseline BioMCP usage.

### OLS4

- BioMCP surfaces: `discover <query>`
- Integration mode: `direct_api`
- BioMCP auth: `none`
- Provider access / registration: open public ontology service
- License / terms summary: EMBL-EBI ontology browser; each ontology keeps its own license
- Redistribution / reuse summary: ontology metadata is queryable, but downstream reuse depends on the specific ontology surfaced
- Official terms URL: <https://www.ebi.ac.uk/ols4/>
- Reviewed on: `2026-03-20`
- Notes: OLS4 is BioMCP's required backbone for discover. It is an ontology index, not a single-license data source.

### OpenFDA

- BioMCP surfaces: `search adverse-event; get drug <name> label; get drug <name> approvals`
- Integration mode: `direct_api`
- BioMCP auth: `optional_env` via `OPENFDA_API_KEY`
- Provider access / registration: open public API; optional key increases quota headroom
- License / terms summary: FDA-origin public data and API terms
- Redistribution / reuse summary: data is broadly reusable, but avoid implying FDA endorsement and preserve source context
- Official terms URL: <https://open.fda.gov/apis/authentication/>
- API key / account URL: <https://open.fda.gov/apis/authentication/>
- Reviewed on: `2026-03-20`
- Notes: The authentication page says API keys are required while also publishing no-key quotas. BioMCP documents the real runtime behavior: baseline use works without a key, with higher quotas when one is configured.

### OpenTargets

- BioMCP surfaces: `get gene <symbol> diseases; get drug <name> targets; get disease <id> genes`
- Integration mode: `direct_api`
- BioMCP auth: `none`
- Provider access / registration: open public GraphQL API
- License / terms summary: Open Targets data is CC0; platform code is Apache 2.0
- Redistribution / reuse summary: platform data is dedicated to the public domain, but linked evidence still carries source provenance
- Official terms URL: <https://platform-docs.opentargets.org/licence>
- Reviewed on: `2026-03-20`
- Notes: The licence page also lists the licensing status of major upstream datasets consumed by Open Targets.

### PMC OA

- BioMCP surfaces: `get article <id> fulltext`
- Integration mode: `direct_api`
- BioMCP auth: `optional_env` via `NCBI_API_KEY`
- Provider access / registration: open public utility for the PMC Open Access subset
- License / terms summary: open-access subset only; article licenses vary within PMC OA
- Redistribution / reuse summary: full text is reusable only according to each article's specific PMC Open Access license
- Official terms URL: <https://pmc.ncbi.nlm.nih.gov/tools/openftlist/>
- API key / account URL: <https://www.ncbi.nlm.nih.gov/account/settings/>
- Reviewed on: `2026-03-20`
- Notes: BioMCP queries PMC OA on demand and does not ship the article corpus. Returned full text is still governed by article-level licenses.

### PubTator3

- BioMCP surfaces: `search article; get article <pmid> annotations`
- Integration mode: `direct_api`
- BioMCP auth: `optional_env` via `NCBI_API_KEY`
- Provider access / registration: open public API; optional My NCBI API key improves throughput
- License / terms summary: NCBI/NLM public-domain annotation service
- Redistribution / reuse summary: results are broadly reusable, but preserve PMID/source provenance and article-level rights separately
- Official terms URL: <https://www.ncbi.nlm.nih.gov/research/pubtator3/api>
- API key / account URL: <https://www.ncbi.nlm.nih.gov/account/settings/>
- Reviewed on: `2026-03-20`
- Notes: BioMCP uses PubTator3 for article search fan-out and article annotation. `NCBI_API_KEY` is optional quota uplift only.

### QuickGO

- BioMCP surfaces: `get gene <symbol> go`
- Integration mode: `direct_api`
- BioMCP auth: `none`
- Provider access / registration: open public API
- License / terms summary: GO/EMBL-EBI open data service
- Redistribution / reuse summary: query results are generally reusable; preserve GO/EMBL-EBI attribution where expected
- Official terms URL: <https://www.ebi.ac.uk/QuickGO/>
- Reviewed on: `2026-03-20`
- Notes: QuickGO exposes GO data and annotations; some embedded evidence sources can carry their own provenance requirements.

### Reactome

- BioMCP surfaces: `search pathway; get pathway <id>; get gene <symbol> pathways`
- Integration mode: `direct_api`
- BioMCP auth: `none`
- Provider access / registration: open public API
- License / terms summary: Reactome pathway content is CC BY 4.0, with some data exports additionally placed under CC0
- Redistribution / reuse summary: reuse is allowed with attribution; preserve pathway/source provenance in downstream materials
- Official terms URL: <https://reactome.org/license>
- Reviewed on: `2026-03-20`
- Notes: Reactome announced in 2017 that some annotation files moved to CC0 while core site/code materials remained under CC BY 4.0.

### STRING

- BioMCP surfaces: `get gene <symbol> interactions; get protein <id> interactions`
- Integration mode: `direct_api`
- BioMCP auth: `none`
- Provider access / registration: open public API
- License / terms summary: CC BY 4.0
- Redistribution / reuse summary: reuse is allowed with attribution to STRING and the original publication/resource
- Official terms URL: <https://string-db.org/cgi/access?footer_active_subpage=licensing>
- Reviewed on: `2026-03-20`
- Notes: BioMCP queries STRING network endpoints on demand and does not package STRING interaction datasets.

### UniProt

- BioMCP surfaces: `get protein <id>; get gene <symbol> protein`
- Integration mode: `direct_api`
- BioMCP auth: `none`
- Provider access / registration: open public API
- License / terms summary: CC BY 4.0
- Redistribution / reuse summary: reuse is allowed with attribution; linked cross-references can have their own terms
- Official terms URL: <https://www.uniprot.org/help/license>
- Reviewed on: `2026-03-20`
- Notes: BioMCP also surfaces UniProt cross-references to PDB and AlphaFold DB rather than mirroring those datasets directly.

### WikiPathways

- BioMCP surfaces: `search pathway; get pathway <id>`
- Integration mode: `direct_api`
- BioMCP auth: `none`
- Provider access / registration: open public API
- License / terms summary: CC0
- Redistribution / reuse summary: pathway content is dedicated to the public domain; attribution is still good scholarly practice
- Official terms URL: <https://classic.wikipathways.org/index.php/WikiPathways:License_Terms>
- Reviewed on: `2026-03-20`
- Notes: The current license statement is still hosted on the WikiPathways classic site.

## Tier 2 - Credential, account, or license required for the BioMCP feature

### AlphaGenome

- BioMCP surfaces: `get variant <id> predict`
- Integration mode: `direct_api`
- BioMCP auth: `required_env` via `ALPHAGENOME_API_KEY`
- Provider access / registration: provider-controlled access to the AlphaGenome service
- License / terms summary: custom provider terms; access is gated by Google/DeepMind service controls
- Redistribution / reuse summary: do not assume open redistribution rights for returned prediction outputs
- Official terms URL: <https://deepmind.google/science/alphagenome/>
- API key / account URL: <https://deepmind.google/science/alphagenome/>
- Reviewed on: `2026-03-20`
- Notes: BioMCP calls the hosted gRPC endpoint directly. The public product page is the closest official access reference currently exposed.

### DisGeNET

- BioMCP surfaces: `get gene <symbol> disgenet; get disease <id> disgenet`
- Integration mode: `direct_api`
- BioMCP auth: `required_env` via `DISGENET_API_KEY`
- Provider access / registration: account registration and API key required
- License / terms summary: custom provider terms for API and downloads
- Redistribution / reuse summary: do not assume unrestricted redistribution; use according to the provider account terms
- Official terms URL: <https://www.disgenet.com/>
- API key / account URL: <https://www.disgenet.com/>
- Reviewed on: `2026-03-20`
- Notes: DisGeNET's public site advertises free and commercial plans. BioMCP documents the API as key-gated and treats it as provider-controlled.

### NCI CTS

- BioMCP surfaces: `search trial --source nci; get trial <nct_id> --source nci`
- Integration mode: `direct_api`
- BioMCP auth: `required_env` via `NCI_API_KEY`
- Provider access / registration: API key required
- License / terms summary: custom provider API terms for the NCI Clinical Trials Search API
- Redistribution / reuse summary: query output is usable for search and review, but downstream reuse should follow NCI API terms and record provenance
- Official terms URL: <https://clinicaltrialsapi.cancer.gov/>
- API key / account URL: <https://clinicaltrialsapi.cancer.gov/>
- Reviewed on: `2026-03-20`
- Notes: BioMCP treats NCI CTS as an alternate oncology-focused trial backend, not the default public trial source.

### OncoKB

- BioMCP surfaces: `variant oncokb <id>`
- Integration mode: `direct_api`
- BioMCP auth: `required_env` via `ONCOKB_TOKEN`
- Provider access / registration: registration and provider-controlled API access
- License / terms summary: custom provider terms; academic research access is no-fee but licensed, commercial/clinical use requires a paid license
- Redistribution / reuse summary: do not assume open redistribution rights for OncoKB data or proprietary treatment descriptions
- Official terms URL: <https://faq.oncokb.org/licensing>
- API key / account URL: <https://www.oncokb.org/account/register>
- Reviewed on: `2026-03-20`
- Notes: The FAQ states that programmatic access for academic use still requires registration and provider approval.

### Semantic Scholar

- BioMCP surfaces: `get article <id> tldr; article citations <id>; article references <id>; article recommendations <id>`
- Integration mode: `direct_api`
- BioMCP auth: `required_env` via `S2_API_KEY`
- Provider access / registration: API key required for the BioMCP helper commands and governed by the API license agreement
- License / terms summary: custom API license agreement
- Redistribution / reuse summary: the API license restricts repackaging, resale, and broad commercial redistribution without expanded licensing
- Official terms URL: <https://www.semanticscholar.org/product/api/license>
- API key / account URL: <https://www.semanticscholar.org/product/api>
- Reviewed on: `2026-03-20`
- Notes: The overview page says many endpoints are publicly reachable without auth, but BioMCP's helper commands deliberately require `S2_API_KEY` to stay within the supported quota path.

### UMLS

- BioMCP surfaces: `discover <query> crosswalk enrichment`
- Integration mode: `direct_api`
- BioMCP auth: `required_env` via `UMLS_API_KEY`
- Provider access / registration: UTS account plus acceptance of the UMLS Metathesaurus license
- License / terms summary: custom UMLS Metathesaurus license and terminology-specific appendices
- Redistribution / reuse summary: do not assume unrestricted redistribution; some embedded vocabularies add their own restrictions or affiliate licenses
- Official terms URL: <https://www.nlm.nih.gov/databases/umls.html>
- API key / account URL: <https://uts.nlm.nih.gov/uts/signup-login>
- Reviewed on: `2026-03-20`
- Notes: The UMLS landing page explicitly states that you must accept the license and create a UTS account for access.

## Tier 3 - Open or queryable, but with notable terms

### cBioPortal

- BioMCP surfaces: `get variant <id> cohort; study download; study query`
- Integration mode: `direct_api`
- BioMCP auth: `none`
- Provider access / registration: public API access for open studies
- License / terms summary: public API with study-specific downstream terms
- Redistribution / reuse summary: reuse depends on the specific study or consortium behind each dataset
- Official terms URL: <https://www.cbioportal.org/>
- Reviewed on: `2026-03-20`
- Notes: BioMCP uses the public API. TCGA-derived studies are broadly open, while some cohorts such as AACR Project GENIE carry additional access or reuse conditions.

### gnomAD

- BioMCP surfaces: `get gene <symbol> constraint; get variant <id> population`
- Integration mode: `direct_api`
- BioMCP auth: `none`
- Provider access / registration: open public browser/API
- License / terms summary: Broad Institute data policies with attribution and service-specific conditions
- Redistribution / reuse summary: querying is open, but users should review the gnomAD policies before bulk reuse or republishing
- Official terms URL: <https://gnomad.broadinstitute.org/policies>
- Reviewed on: `2026-03-20`
- Notes: Open Targets currently reports gnomAD as CC0 for its own ingestion pipeline. BioMCP links to gnomAD's own policy page for user-facing guidance.

### Human Protein Atlas

- BioMCP surfaces: `get gene <symbol> hpa`
- Integration mode: `direct_api`
- BioMCP auth: `none`
- Provider access / registration: open public XML and web resources
- License / terms summary: CC BY-SA 4.0 for copyrightable parts of the database
- Redistribution / reuse summary: reuse is allowed with attribution and ShareAlike; third-party components may impose extra conditions
- Official terms URL: <https://www.proteinatlas.org/about/licence>
- Reviewed on: `2026-03-20`
- Notes: The licence page also requires clear citation for images and specific gene/data pages.

### KEGG

- BioMCP surfaces: `search pathway; get pathway <id>`
- Integration mode: `direct_api`
- BioMCP auth: `none`
- Provider access / registration: open website/API for academic use; commercial use requires a license
- License / terms summary: custom KEGG terms; academic users may freely use the website, non-academic use requires a commercial license
- Redistribution / reuse summary: do not assume commercial redistribution rights; query access does not grant a redistribution license
- Official terms URL: <https://www.kegg.jp/kegg/legal.html>
- Reviewed on: `2026-03-20`
- Notes: KEGG's official legal page was updated on October 1, 2024 and explicitly distinguishes academic from non-academic use.

### PharmGKB

- BioMCP surfaces: `get pgx <gene_or_drug> annotations`
- Integration mode: `direct_api`
- BioMCP auth: `none`
- Provider access / registration: open public API
- License / terms summary: ClinPGx API data is CC BY-SA 4.0 and subject to the provider's data usage policy
- Redistribution / reuse summary: reuse is allowed with attribution and ShareAlike; some underlying annotations and external assets may add extra constraints
- Official terms URL: <https://api.pharmgkb.org/>
- Reviewed on: `2026-03-20`
- Notes: PharmGKB has been transitioning to ClinPGx-branded API documentation. BioMCP keeps the public `PharmGKB` source label because that is the domain vocabulary users recognize.

## Indirect-only providers surfaced through aggregators

### AlphaFold DB

- BioMCP surfaces: `get protein <id> structures`
- Integration mode: `indirect_only`
- BioMCP auth: `not_applicable`
- Provider access / registration: surfaced through UniProt cross-references; no standalone BioMCP client
- License / terms summary: AlphaFold DB structural predictions are published for broad open use
- Redistribution / reuse summary: reuse is generally open, but preserve model/source provenance and article citations
- Official terms URL: <https://alphafold.ebi.ac.uk/faq>
- Reviewed on: `2026-03-20`
- Notes: BioMCP does not call AlphaFold DB directly. Structure links appear through UniProt cross-references.

### Cancer Genome Interpreter

- BioMCP surfaces: `get variant <id>`
- Integration mode: `indirect_only`
- BioMCP auth: `not_applicable`
- Provider access / registration: surfaced indirectly through MyVariant.info payloads
- License / terms summary: custom tool terms
- Redistribution / reuse summary: do not assume commercial reuse rights; the official terms restrict some external and commercial use
- Official terms URL: <https://www.cancergenomeinterpreter.org/conditions>
- Reviewed on: `2026-03-20`
- Notes: There is no standalone CGI source client in BioMCP; provenance appears only when MyVariant includes CGI fields.

### ClinVar

- BioMCP surfaces: `get variant <id> clinvar`
- Integration mode: `indirect_only`
- BioMCP auth: `not_applicable`
- Provider access / registration: surfaced indirectly through MyVariant.info payloads
- License / terms summary: NCBI public-domain submission archive
- Redistribution / reuse summary: records are broadly reusable, but preserve accession/provenance and submitter context
- Official terms URL: <https://www.ncbi.nlm.nih.gov/clinvar/docs/maintenance_use/>
- Reviewed on: `2026-03-20`
- Notes: BioMCP does not call ClinVar directly; ClinVar assertions arrive through MyVariant.info.

### COSMIC

- BioMCP surfaces: `get variant <id> cosmic`
- Integration mode: `indirect_only`
- BioMCP auth: `not_applicable`
- Provider access / registration: surfaced indirectly through cached MyVariant.info fields; no standalone BioMCP COSMIC client
- License / terms summary: custom COSMIC licensing with commercial restrictions
- Redistribution / reuse summary: direct redistribution and direct integration remain intentionally unsupported without a separate COSMIC license
- Official terms URL: <https://www.sanger.ac.uk/legal/cosmic-licensing/>
- Reviewed on: `2026-03-20`
- Notes: This is the most important indirect-only caution row. BioMCP intentionally does not support direct COSMIC querying because of licensing risk.

### Disease Ontology

- BioMCP surfaces: `search disease; get disease <id>`
- Integration mode: `indirect_only`
- BioMCP auth: `not_applicable`
- Provider access / registration: surfaced indirectly through MyDisease.info
- License / terms summary: open disease ontology project
- Redistribution / reuse summary: reuse is generally open; preserve ontology version and source references
- Official terms URL: <https://disease-ontology.org/>
- Reviewed on: `2026-03-20`
- Notes: BioMCP does not maintain a standalone Disease Ontology client.

### DrugBank

- BioMCP surfaces: `get drug <name> interactions; get drug <name> label`
- Integration mode: `indirect_only`
- BioMCP auth: `not_applicable`
- Provider access / registration: surfaced indirectly through MyChem.info payloads
- License / terms summary: custom DrugBank terms of use and licensing
- Redistribution / reuse summary: use or redistribution of DrugBank content requires a DrugBank license; do not assume open downstream rights
- Official terms URL: <https://trust.drugbank.com/drugbank-trust-center/drugbank-terms-of-service>
- Reviewed on: `2026-03-20`
- Notes: DrugBank does not have a standalone BioMCP source client. It appears as provenance carried through MyChem.info.

### Drugs@FDA

- BioMCP surfaces: `get drug <name> approvals`
- Integration mode: `indirect_only`
- BioMCP auth: `not_applicable`
- Provider access / registration: surfaced through OpenFDA-derived approval fields
- License / terms summary: FDA-origin public information
- Redistribution / reuse summary: approval records are broadly reusable; avoid implying FDA endorsement
- Official terms URL: <https://open.fda.gov/apis/drug/drugsfda/>
- Reviewed on: `2026-03-20`
- Notes: BioMCP documents Drugs@FDA as an indirect provenance label because approval fields arrive through OpenFDA, not a dedicated Drugs@FDA client.

### MONDO

- BioMCP surfaces: `search disease; discover <query>`
- Integration mode: `indirect_only`
- BioMCP auth: `not_applicable`
- Provider access / registration: surfaced indirectly through MyDisease.info payloads
- License / terms summary: CC BY 4.0
- Redistribution / reuse summary: reuse is allowed with attribution and ontology version tracking
- Official terms URL: <https://mondo.monarchinitiative.org/pages/download/>
- Reviewed on: `2026-03-20`
- Notes: BioMCP uses MONDO identifiers through MyDisease.info and other aggregators rather than calling MONDO directly.

### PDB

- BioMCP surfaces: `get protein <id> structures`
- Integration mode: `indirect_only`
- BioMCP auth: `not_applicable`
- Provider access / registration: surfaced through UniProt cross-references; no standalone BioMCP PDB client
- License / terms summary: PDB archive data is CC0 1.0
- Redistribution / reuse summary: data is broadly reusable; attribution to original structure authors is encouraged
- Official terms URL: <https://www.rcsb.org/pages/usage-policy>
- Reviewed on: `2026-03-20`
- Notes: BioMCP currently exposes PDB identifiers from UniProt rather than querying RCSB PDB directly.

## Source notes

- `PubMed` is an umbrella label in BioMCP output. In practice, article search and annotation use `PubTator3`, `Europe PMC`, `PMC OA`, and `NCBI ID Converter`. This page maps those aliases back to the actual provider rows instead of creating duplicate inventory entries.
- `OpenFDA FAERS`, `OpenFDA label`, `OpenFDA shortage`, and `Drugs@FDA` are user-facing provenance labels that resolve back to the `OpenFDA` direct row plus the `Drugs@FDA` indirect row.
- `AlphaFold DB` and `PDB` are indirect-only because BioMCP currently surfaces those structure IDs via `UniProt` cross-references rather than maintaining standalone source clients.
- `COSMIC` is indirect-only provenance through `MyVariant.info`. Direct COSMIC integration is not part of BioMCP's supported source surface because the provider's licensing model creates unacceptable redistribution and deployment risk for an MIT-licensed open tool.
