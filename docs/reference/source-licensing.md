# Source Licensing and Terms

BioMCP itself is MIT-licensed, but the data providers it queries are not all licensed the same way.

Captured provider responses under `testdata/` are repository-only verification
inputs. BioMCP excludes them from crates.io source packages and public build
artifacts because redistribution rights have not been established for every
provider. This is a precautionary package boundary, not a claim that keeping
the fixtures in the public source repository is improper. Redistributing any
specific capture requires a separate source-by-source rights record.

Three distinctions matter:

- BioMCP does not vendor, mirror, or ship upstream datasets in the repository.
- BioMCP performs on-demand read-only queries against upstream services.
- Returned records, downloaded full text, saved output, and downstream reuse can still be governed by upstream provider terms.

Use this page for provider terms, licensing, redistribution, and account requirements. Use [API Keys](../getting-started/api-keys.md) for setup steps and [Data Sources](data-sources.md) for runtime behavior, endpoints, rate limits, and operational caveats.

The canonical machine-readable inventory for this page lives in [`sources.json`](sources.json). Update its facts and notes, then run `python3 tools/render-source-licensing.py --write` to refresh this page. Generated blocks include each source's notes; keep source-specific terms and caveats in the registry.

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
- `commercial_terms`
  `null` means the recorded terms impose no recorded commercial restriction on the content BioMCP serves.
  `license-required` means the recorded terms require a commercial license for commercial use.
  `non-commercial` means the recorded terms allow non-commercial use only.
  `restricted` means the recorded terms restrict commercial use without establishing either narrower category.

### Turning restricted sources off

The `commercial_terms` field draws the `--disable-sources` groups:
`license-required` and `non-commercial` each select their own shape, and
`commercial-restricted` includes both groups and sources whose recorded terms restrict commercial use without specifying either narrower category. Turn any of them off with the global
flag or the environment variable
([Configuration: disabling sources by name or group](configuration.md#disabling-sources-by-name-or-group));
any single source id also works on its own. Membership follows this file, so
re-recording a source's terms redraws the groups without a code change.
Current group membership comes from the registry:

<!-- licensing:groups -->
- `license-required`: DisGeNET, DrugBank, Enrichr, KEGG, OncoKB.
- `non-commercial`: AlphaGenome, DDInter, ORCID, PharmacoDB.
- `commercial-restricted`: AlphaGenome, Cancer Genome Interpreter, DDInter, DisGeNET, DrugBank, Enrichr, KEGG, OncoKB, ORCID, PharmacoDB.

<!-- /licensing -->

## Summary table

<!-- licensing:summary -->
| Source | Tier | Mode | BioMCP auth | License / terms summary | Redistribution summary | Terms URL |
|---|---|---|---|---|---|---|
| AlphaFold DB | 1 | indirect_only | not_applicable | AlphaFold DB structural predictions are published for broad open use | reuse is generally open, but preserve model/source provenance and article citations | <https://alphafold.ebi.ac.uk/faq> |
| AlphaGenome | 2 | direct_api | required_env | restricted: the hosted AlphaGenome API permits non-commercial use under provider terms | hosted API outputs are restricted to non-commercial research; provider exceptions for downloadable artifacts and separate cloud commercial access do not grant this API integration those rights | <https://google-deepmind-alphagenome.readthedocs-hosted.com/faqs.html#what-terms-of-use-apply-to-alphagenome-outputs> |
| Cancer Genome Interpreter | 3 | indirect_only | not_applicable | restricted: custom tool terms restrict external and commercial use | do not assume commercial reuse rights; the official terms restrict some external and commercial use | <https://www.cancergenomeinterpreter.org/conditions> |
| Cancerhotspots.org | 3 | direct_api | none | public cancer hotspot recurrence resource with provider-specific reuse expectations | BioMCP queries recurrence counts live and does not redistribute a local cancerhotspots dataset | <https://www.cancerhotspots.org/> |
| cBioPortal | 3 | direct_api | none | public API with study-specific downstream terms | reuse depends on the specific study or consortium behind each dataset | <https://www.cbioportal.org/> |
| CDC CVX/MVX | 1 | direct_api | none | most CDC website materials are public domain, but attribution, disclaimer, and exceptions for third-party or non-federal content still apply | reuse is generally allowed with CDC attribution and non-endorsement language; avoid CDC logos and review exceptions before republishing | <https://www.cdc.gov/other/agencymaterials.html> |
| Cell Model Passports | 1 | indirect_only | not_applicable | Sanger DepMap Data Usage Policy: a non-exclusive, non-transferable right to use the data files for internal proprietary research and education; resale of all or a significant part and provision of commercial services are excluded, and commercial API/data use requires prior consent (non-commercial API access needs no registration) | BioMCP shows the identifier through a Cellosaurus cross-reference only; users who follow it to the portal use the portal under its policy | <https://depmap.sanger.ac.uk/documentation/data-usage-policy/> |
| Cellosaurus | 1 | direct_api | none | CC BY 4.0 | reuse is allowed with attribution and the requested citation on reuse | <https://www.cellosaurus.org/description.html> |
| ChEMBL | 1 | direct_api | none | ChEMBL data is provided under Creative Commons Attribution-ShareAlike 3.0 (CC BY-SA 3.0); attribution is required and share-alike applies to adaptations (2026-09-27 pass added the ShareAlike obligation the earlier summary omitted). | reuse, redistribution and adaptation are allowed with attribution under ShareAlike; preserve ChEMBL IDs and name the release, and note that compound property calculations derived from commercial software carry their own terms | <https://www.ebi.ac.uk/chembl/> |
| CIViC | 1 | direct_api | none | open community knowledgebase; CIViC content is published for unrestricted reuse. Verified 2026-09-28 against the live FAQ (plain HTML): CIViC content is released under CC0 1.0 Universal; the source code is MIT; CIViC is intended for purely research purposes. | reuse is broadly permitted; attribution remains best practice | <https://docs.civicdb.org/en/latest/about/faq.html> |
| ClinGen | 1 | direct_api | none | public ClinGen curation resources with publication and attribution expectations | generally queryable and reusable, but users should preserve attribution and source context | <https://clinicalgenome.org/> |
| ClinGen Allele Registry | 1 | direct_api | none | public ClinGen curation resources with publication and attribution expectations | generally queryable and reusable, but users should preserve attribution and source context | <https://reg.genome.network> |
| ClinGen CSpec | 1 | direct_api | none | public ClinGen curation resources with publication and attribution expectations | generally queryable and reusable, but users should preserve attribution and source context | <https://cspec.clinicalgenome.org> |
| ClinGen ERepo | 1 | direct_api | none | public ClinGen curation resources with publication and attribution expectations | generally queryable and reusable, but users should preserve attribution and source context | <https://erepo.clinicalgenome.org> |
| ClinGen LDH | 1 | direct_api | none | public ClinGen curation resources with publication and attribution expectations | generally queryable and reusable, but users should preserve attribution and source context | <https://ldh.genome.network> |
| ClinicalTrials.gov | 1 | direct_api | none | U.S. government public information service | records are broadly reusable; preserve identifiers and avoid implying NLM endorsement | <https://clinicaltrials.gov/data-api/about-api> |
| ClinVar | 1 | direct_api | optional_env | NCBI public-domain submission archive | records are broadly reusable, but preserve accession/provenance and submitter context | <https://www.ncbi.nlm.nih.gov/clinvar/docs/maintenance_use/> |
| ComplexPortal | 1 | direct_api | none | EMBL-EBI open data service | reuse follows EMBL-EBI resource terms and any embedded third-party source obligations | <https://www.ebi.ac.uk/complexportal/> |
| COSMIC | 3 | indirect_only | not_applicable | BioMCP reads COSMIC only through MyVariant.info, whose COSMIC fields are the v68 snapshot — per MyVariant's metadata: 'COSMIC v68 was imported from UCSC database dump. This is the last freely available somatic variants from COSMIC before their licence change' (checked 2026-09-27). The v68-era terms, not the current cosmickb.org licence, govern those fields. The current licence requires a commercial licence for commercial use including patient services and clinical reporting; it governs COSMIC's current direct offering only. The practical limit of the v68 fields is age, not licence. | MyVariant's COSMIC v68 snapshot predates the current COSMIC licence; treat the fields under the v68-era terms and cite MyVariant as the serving source. Direct use of current COSMIC releases remains commercially restricted. | <https://docs.myvariant.info/en/latest/doc/data_source.html> |
| CPIC | 1 | direct_api | none | cpicpgx.org now redirects to ClinPGx and its former CC0 licence pages are gone; the successor data-usage policy states ClinPGx data is CC BY-SA 4.0. Whether that licence governs CPIC content is not stated on any live page; treat CPIC data as attribution-plus-share-alike until CPIC publishes terms on its own domain again. | content reuse is broadly allowed, but the CPIC mark/logo has separate restrictions | <https://www.clinpgx.org/page/dataUsagePolicy> |
| dbSNP | 1 | direct_api | none | NCBI public data service with attribution and non-endorsement expectations | RefSNP records are publicly queryable; preserve dbSNP provenance and do not imply NCBI endorsement | <https://www.ncbi.nlm.nih.gov/home/about/policies/> |
| DDInter | 3 | direct_api | none | CC BY-NC-SA 4.0 with an explicit completeness disclaimer; absence from the database does not prove no interaction exists | reuse requires attribution, non-commercial use, and ShareAlike treatment; do not turn missing rows into safety claims or assume commercial redistribution rights | <https://ddinter.scbdd.com/terms/> |
| DepMap | 1 | indirect_only | not_applicable | DepMap's own guidance says DepMap-generated data are generally CC BY 4.0 while other hosted projects may use different licenses; each file's Data Usage and Terms must be checked per download, and a 2026-07-28 staff update reports newly generated data carry restricted terms (research use free, commercial use separately licensed) | BioMCP shows the identifier through a Cellosaurus cross-reference only; no DepMap bytes enter BioMCP output beyond the identifier | <https://forum.depmap.org/t/license-for-data-found-in-the-depmap-portal/130> |
| DGIdb | 1 | direct_api | none | open interaction service; aggregated claims may still reflect upstream source terms | treat DGIdb as an aggregation layer and preserve source attribution for underlying claim providers | <https://www.dgidb.org/about> |
| Disease Ontology | 1 | indirect_only | not_applicable | open disease ontology project | reuse is generally open; preserve ontology version and source references | <https://disease-ontology.org/> |
| DisGeNET | 2 | direct_api | required_env | restricted: commercial use requires a commercial license | standard licenses prohibit redistribution and resale of all or part of the database; custom permission is required | <https://support.disgenet.com/support/solutions/articles/202000087487-do-i-need-a-commercial-license-can-i-use-disgenet-data-in-my-product-> |
| DrugBank | 3 | indirect_only | not_applicable | custom DrugBank terms of use and licensing | use or redistribution of DrugBank content requires a DrugBank license; do not assume open downstream rights | <https://trust.drugbank.com/drugbank-trust-center/drugbank-terms-of-service> |
| Drugs@FDA | 1 | indirect_only | not_applicable | FDA-origin public information | approval records are broadly reusable; avoid implying FDA endorsement | <https://open.fda.gov/apis/drug/drugsfda/> |
| EMA | 1 | direct_api | none | EMA website material may be reused with source attribution; third-party content can carry separate rights | EMA-published website data is generally reusable with attribution, but embedded third-party materials may need separate permission | <https://www.ema.europa.eu/en/about-us/about-website/legal-notice> |
| Enrichr | 3 | direct_api | none | Verified 2026-09-29 against the live terms page: free for academic and non-profit use; commercial use requires a license from Mount Sinai Innovation Partners. The terms state Enrichr is not to be used for treating or diagnosing human subjects. | Commercial use requires a license from Mount Sinai Innovation Partners; not for treating or diagnosing human subjects. Users are responsible for their own licenses. | <https://maayanlab.cloud/Enrichr/templates/help/terms-submenu.html> |
| Europe PMC | 1 | direct_api | none | open literature metadata service; article and full-text licenses vary by record | metadata is broadly reusable, but full text and PDFs remain governed by article-level licenses | <https://europepmc.org/RestfulWebService> |
| FDA Orphan Drug Designations and Approvals | 1 | direct_api | none | FDA-origin public information | designation records are broadly reusable; preserve FDA source context and never imply designation is approval | <https://www.fda.gov/about-fda/about-website/website-policies> |
| Figshare | 1 | direct_api | none | public repository API; each article/file carries its own license metadata | reuse downloaded article assets according to the Figshare item license and preserve Figshare/article provenance | <https://figshare.com/terms> |
| GenCC | 1 | direct_api | none | CC0 1.0 data with requested attribution to GenCC and contributing submitters | reuse is permitted under CC0; preserve GenCC and submitter attribution and the non-diagnostic context | <https://thegencc.org/terms> |
| gnomAD | 3 | direct_api | none | Broad Institute data policies with attribution and service-specific conditions | querying is open, but users should review the gnomAD policies before bulk reuse or republishing | <https://gnomad.broadinstitute.org/policies> |
| g:Profiler | 1 | direct_api | none | open enrichment service with provider citation expectations | results are queryable and reusable, but cite g:Profiler and any underlying databases you depend on | <https://biit.cs.ut.ee/gprofiler/help.cgi> |
| GTEx | 1 | direct_api | none | NIH-hosted public-access expression resource | public summary/expression views are broadly reusable; controlled-access data remains outside BioMCP's scope | <https://gtexportal.org/home/documentationPage> |
| NCBI Genetic Testing Registry | 1 | direct_api | none | NLM/NCBI government data service with public bulk-export access | GTR records are broadly queryable and reusable, but preserve accession provenance and review any embedded third-party identifiers separately | <https://www.ncbi.nlm.nih.gov/gtr/docs/faq/> |
| GWAS Catalog | 1 | direct_api | none | EMBL-EBI resource terms; summary statistics may carry separate licenses | query results are generally reusable, but dataset-level summary statistics can have separate downstream terms | <https://www.ebi.ac.uk/gwas/docs/about> |
| HPO JAX API | 1 | direct_api | none | open HPO data with attribution and integrity requirements | reuse is allowed, but users should preserve attribution, version context, and source integrity | <https://human-phenotype-ontology.github.io/license.html> |
| Human Protein Atlas | 3 | direct_api | none | CC BY 4.0 for copyrightable parts of the database | reuse is allowed with attribution; third-party components may impose extra conditions | <https://www.proteinatlas.org/about/licence> |
| InterPro | 1 | direct_api | none | EMBL-EBI open data resource | reuse follows InterPro/EMBL-EBI resource terms and any embedded member-database obligations | <https://www.ebi.ac.uk/about/terms-of-use> |
| KEGG | 3 | direct_api | none | custom KEGG terms; academic users may freely use the website, non-academic use requires a commercial license | do not assume commercial redistribution rights; query access does not grant a redistribution license | <https://www.kegg.jp/kegg/legal.html> |
| LitSense2 | 1 | direct_api | none | NCBI/NLM public-domain literature service | query results are broadly reusable, but preserve article-level provenance and record rights separately | <https://www.ncbi.nlm.nih.gov/research/litsense2-api/> |
| LitVar2 | 1 | direct_api | none | NCBI/NLM public-domain literature service; variant-literature links are free to query | query results are broadly reusable, but each paper's own reuse rights stay separate | <https://www.ncbi.nlm.nih.gov/research/litvar2-api/> |
| MedlinePlus | 1 | direct_api | none | NLM public-information service with trademark and endorsement guidance | content is widely reusable, but preserve attribution and avoid implying MedlinePlus/NLM endorsement | <https://medlineplus.gov/about/using/> |
| Monarch Initiative | 1 | direct_api | none | open integrated knowledge graph; underlying source licenses still matter | results can be queried openly, but downstream reuse should respect the original sources folded into Monarch | <https://monarchinitiative.org/> |
| MONDO | 1 | indirect_only | not_applicable | CC BY 4.0 | reuse is allowed with attribution and ontology version tracking | <https://mondo.monarchinitiative.org/pages/download/> |
| Mutalyzer | 1 | direct_api | none | public Mutalyzer normalization service; provider terms apply to returned validation output | preserve Mutalyzer source identity and returned HGVS/protein fields when reusing results | <https://mutalyzer.nl/> |
| MyChem.info | 1 | direct_api | none | BioThings aggregation service; upstream source terms continue to apply | do not assume aggregator responses are relicensed; preserve source provenance for downstream reuse. Shared BioThings footer: intended for research purposes; not for emergencies or medical advice (noted 2026-09-27 pass). | <https://docs.mychem.info/en/latest/> |
| MyDisease.info | 1 | direct_api | none | BioThings aggregation service; source ontologies and datasets keep their own terms | treat payloads as aggregated source data rather than a new umbrella license. Shared BioThings footer: intended for research purposes; not for emergencies or medical advice (noted 2026-09-27 pass). | <https://docs.mydisease.info/en/latest/> |
| MyGene.info | 1 | direct_api | none | BioThings aggregation service; source-specific terms remain attached to underlying records | reuse should preserve provenance back to NCBI Gene, UniProt, and other upstream sources. Shared BioThings footer: intended for research purposes; not for emergencies or medical advice (added 2026-09-27 pass). | <https://docs.mygene.info/en/latest/> |
| MyVariant.info | 1 | direct_api | none | BioThings aggregation service; indirect providers retain their own terms | ClinVar, Cancer Genome Interpreter, and gnomAD-related fields follow their original providers' terms; the COSMIC fields are the v68 pre-licence-change snapshot per MyVariant's own metadata, so the current COSMIC licence does not attach to them. Shared BioThings footer: intended for research purposes; not for emergencies or medical advice (noted 2026-09-27 pass). | <https://docs.myvariant.info/en/latest/> |
| NCBI E-utilities | 1 | direct_api | optional_env | NLM public-domain utility service | utility responses are broadly reusable; article-level full text still follows the returned record's license context | <https://www.ncbi.nlm.nih.gov/books/NBK25501/> |
| NCBI ID Converter | 1 | direct_api | optional_env | NLM public-domain utility service | utility results are broadly reusable; keep article-level identifiers and downstream article licenses distinct | <https://pmc.ncbi.nlm.nih.gov/tools/idconv/> |
| NCI CTS | 2 | direct_api | required_env | custom provider API terms for the NCI Clinical Trials Search API | query output is usable for search and review, but downstream reuse should follow NCI API terms and record provenance | <https://clinicaltrialsapi.cancer.gov/> |
| NIH Reporter | 1 | direct_api | none | NIH-operated public funding-reporting service | project and grant metadata are broadly reusable; preserve NIH Reporter attribution, fiscal-year context, and grant identifiers | <https://www.nih.gov/web-policies-notices> |
| OLS4 | 1 | direct_api | none | EMBL-EBI ontology browser; each ontology keeps its own license | ontology metadata is queryable, but downstream reuse depends on the specific ontology surfaced | <https://www.ebi.ac.uk/about/terms-of-use> |
| OncoKB | 2 | direct_api | required_env | custom provider terms; academic research access is no-fee but licensed, commercial/clinical use requires a paid license | do not assume open redistribution rights for OncoKB data or proprietary treatment descriptions | <https://faq.oncokb.org/licensing> |
| OpenCitations | 1 | direct_api | none | OpenCitations datasets are released under a CC0 public domain dedication; website text is CC BY 4.0 and the software is ISC | citation data is dedicated to the public domain and may be freely reused for any purpose, including commercial use; the OpenCitations trademark policy still governs the name | <https://opencitations.net/> |
| OpenFDA | 1 | direct_api | optional_env | FDA-origin public data and API terms | data is broadly reusable, but avoid implying FDA endorsement and preserve source context | <https://open.fda.gov/apis/authentication/> |
| OpenTargets | 1 | direct_api | none | Open Targets data is CC0; platform code is Apache 2.0 | platform data is dedicated to the public domain, but linked evidence still carries source provenance | <https://platform-docs.opentargets.org/licence> |
| ORCID | 2 | direct_api | required_env | restricted: ORCID Public and Anonymous APIs permit non-commercial use; section 2 excludes use in revenue-generating products or services even though public record data is CC0 | display public record data with attribution; do not assume all record data or downstream works are identically licensed | <https://info.orcid.org/public-client-terms-of-service/> |
| PDB | 1 | indirect_only | not_applicable | PDB archive data is CC0 1.0 | data is broadly reusable; attribution to original structure authors is encouraged | <https://www.rcsb.org/pages/usage-policy> |
| PharmacoDB | 3 | direct_api | none | PharmacoDB publishes no licence or terms page; its source code is GPL-3.0 and the describing paper is CC BY-NC 4.0, and the terms for the data itself are unstated by the provider | treat reuse as non-commercial and attribute PharmacoDB; the provider states no terms for the data, so redistribution rights are not established | none published |
| PharmGKB | 3 | direct_api | none | PharmGKB's own domain now redirects to ClinPGx and the former terms pages are gone; the successor data-usage policy (read from the live page's JS bundle) states ClinPGx data is CC BY-SA 4.0. Verified 2026-09-28 against the same page CPIC was, since both routes lead to one policy. | reuse is allowed with attribution and ShareAlike; some underlying annotations and external assets may add extra constraints | <https://www.clinpgx.org/page/dataUsagePolicy> |
| PMC OA | 1 | direct_api | optional_env | open-access subset only; article licenses vary within PMC OA | full text is reusable only according to each article's specific PMC Open Access license | <https://pmc.ncbi.nlm.nih.gov/tools/openftlist/> |
| PubMed | 1 | direct_api | optional_env | NLM public-domain search and metadata service | search results are broadly reusable, but article-level abstracts, full text, and downstream reuse still depend on the returned record context | <https://www.ncbi.nlm.nih.gov/books/NBK25501/> |
| PubTator3 | 1 | direct_api | optional_env | NCBI/NLM public-domain annotation service | results are broadly reusable, but preserve PMID/source provenance and article-level rights separately | <https://www.ncbi.nlm.nih.gov/research/pubtator3/api> |
| QuickGO | 1 | direct_api | none | GO/EMBL-EBI open data service | query results are generally reusable; preserve GO/EMBL-EBI attribution where expected | <https://www.ebi.ac.uk/about/terms-of-use> |
| Reactome | 1 | direct_api | none | Reactome database data and derived files are CC0; illustrations, icons, art and branding are CC BY 4.0; software has separate licenses | database data and derived files may be reused commercially without permission; attribution is encouraged for data and required for CC BY 4.0 art | <https://reactome.org/license> |
| SEER Explorer | 1 | direct_api | none | U.S. government cancer-statistics website and public explorer | summary survival statistics are broadly reusable, but users should preserve SEER/NCI attribution and understand the explorer endpoints are undocumented UI routes | <https://seer.cancer.gov/about/using-website.html> |
| Semantic Scholar | 2 | direct_api | optional_env | custom API license agreement | the API license restricts repackaging, resale, and broad commercial redistribution without expanded licensing | <https://www.semanticscholar.org/product/api/license> |
| STRING | 1 | direct_api | none | CC BY 4.0 | reuse is allowed with attribution to STRING and the original publication/resource | <https://string-db.org/cgi/access?footer_active_subpage=licensing> |
| UMLS | 2 | direct_api | required_env | custom UMLS Metathesaurus license and terminology-specific appendices | do not assume unrestricted redistribution; some embedded vocabularies add their own restrictions or affiliate licenses | <https://www.nlm.nih.gov/databases/umls.html> |
| UniProt | 1 | direct_api | none | CC BY 4.0 | reuse is allowed with attribution; linked cross-references can have their own terms | <https://www.uniprot.org/help/license> |
| CDC WONDER VAERS | 1 | direct_api | none | CDC WONDER data use restrictions require statistical reporting/analysis use and prohibit re-identification attempts | reuse is allowed for statistical reporting and analysis with source attribution, but do not try to identify individuals or publish identifying linkages | <https://wonder.cdc.gov/datause.html> |
| VariantValidator | 1 | direct_api | none | public VariantValidator service; provider terms apply to returned validation output | preserve VariantValidator source identity, warnings, and genomic descriptions when reusing results | <https://variantvalidator.org/> |
| WHO Prequalified IVD | 1 | direct_api | none | WHO content generally requires attribution; commercial reuse may require permission and third-party materials can carry separate rights | Preserve WHO attribution, review commercial-use conditions, and verify third-party material rights before republishing WHO IVD-derived content | <https://www.who.int/about/policies/publishing/copyright> |
| WHO Prequalification | 1 | direct_api | none | WHO content generally requires attribution; commercial reuse may require permission and third-party materials can carry separate rights | preserve WHO attribution, review commercial-use conditions, and check third-party material rights before republishing | <https://www.who.int/about/policies/publishing/copyright> |
| WikiPathways | 1 | direct_api | none | Verified 2026-09-28 against the live terms page: WikiPathways adopts the Creative Commons CC0 waiver for its content (the page says so in plain text). The earlier unverifiable call reflected the retired classic site; the current site's terms page is live. | pathway content is dedicated to the public domain; attribution is still good scholarly practice | <https://www.wikipathways.org/terms.html> |

<!-- /licensing -->

## Tier 1 - Baseline use without credentials

<!-- licensing:alphafold-db -->
### AlphaFold DB

- BioMCP surfaces: `get protein <id> structures`
- Integration mode: `indirect_only`
- BioMCP auth: `not_applicable`
- Commercial terms: `null`
- Via: UniProt
- Provider access / registration: surfaced through UniProt cross-references; no standalone BioMCP client
- License / terms summary: AlphaFold DB structural predictions are published for broad open use
- Redistribution / reuse summary: reuse is generally open, but preserve model/source provenance and article citations
- Official terms URL: <https://alphafold.ebi.ac.uk/faq>
- Reviewed on: `2026-09-27`
- Notes: BioMCP does not call AlphaFold DB directly. Structure links appear through UniProt cross-references.

<!-- /licensing -->

<!-- licensing:clinvar -->
### ClinVar

- BioMCP surfaces: `get variant <id> clinvar`
- Integration mode: `direct_api`
- BioMCP auth: `optional_env`
- Commercial terms: `null`
- Credential environment variable: `NCBI_API_KEY`
- Provider access / registration: open public NCBI E-utilities API; MyVariant.info is the fallback carrier
- License / terms summary: NCBI public-domain submission archive
- Redistribution / reuse summary: records are broadly reusable, but preserve accession/provenance and submitter context
- Official terms URL: <https://www.ncbi.nlm.nih.gov/clinvar/docs/maintenance_use/>
- API key / account URL: <https://www.ncbi.nlm.nih.gov/account/settings/>
- Reviewed on: `2026-09-27`
- Notes: Explicit clinvar and all variant requests use NCBI EFetch by resolved Variation ID; the default card and degraded fallback use MyVariant.info.

<!-- /licensing -->

<!-- licensing:dbsnp -->
### dbSNP

- BioMCP surfaces: `get variant <id> population`
- Integration mode: `direct_api`
- BioMCP auth: `none`
- Commercial terms: `null`
- Provider access / registration: open public API
- License / terms summary: NCBI public data service with attribution and non-endorsement expectations
- Redistribution / reuse summary: RefSNP records are publicly queryable; preserve dbSNP provenance and do not imply NCBI endorsement
- Official terms URL: <https://www.ncbi.nlm.nih.gov/home/about/policies/>
- Reviewed on: `2026-08-20`
- Notes: BioMCP uses the NCBI Variation RefSNP API only to recover an explicitly labeled GRCh38 chromosome coordinate for eligible identifier-based gnomAD population lookups.

<!-- /licensing -->

<!-- licensing:disease-ontology -->
### Disease Ontology

- BioMCP surfaces: `search disease`; `get disease <id>`
- Integration mode: `indirect_only`
- BioMCP auth: `not_applicable`
- Commercial terms: `null`
- Via: MyDisease.info
- Provider access / registration: surfaced indirectly through MyDisease.info
- License / terms summary: open disease ontology project
- Redistribution / reuse summary: reuse is generally open; preserve ontology version and source references
- Official terms URL: <https://disease-ontology.org/>
- Reviewed on: `2026-09-27`
- Notes: BioMCP does not maintain a standalone Disease Ontology client.

<!-- /licensing -->

<!-- licensing:drugs-at-fda -->
### Drugs@FDA

- BioMCP surfaces: `get drug <name> approvals`
- Integration mode: `indirect_only`
- BioMCP auth: `not_applicable`
- Commercial terms: `null`
- Via: OpenFDA
- Provider access / registration: surfaced through OpenFDA-derived approval fields
- License / terms summary: FDA-origin public information
- Redistribution / reuse summary: approval records are broadly reusable; avoid implying FDA endorsement
- Official terms URL: <https://open.fda.gov/apis/drug/drugsfda/>
- Reviewed on: `2026-09-27`
- Notes: BioMCP documents Drugs@FDA as an indirect provenance label because approval fields arrive through OpenFDA, not a dedicated Drugs@FDA client.

<!-- /licensing -->

<!-- licensing:litsense2 -->
### LitSense2

- BioMCP surfaces: `search article -k <query>`
- Integration mode: `direct_api`
- BioMCP auth: `none`
- Commercial terms: `null`
- Provider access / registration: open public API
- License / terms summary: NCBI/NLM public-domain literature service
- Redistribution / reuse summary: query results are broadly reusable, but preserve article-level provenance and record rights separately
- Official terms URL: <https://www.ncbi.nlm.nih.gov/research/litsense2-api/>
- Reviewed on: `2026-04-10`
- Notes: LitSense2 is individually selectable with search article --source litsense2 and contributes semantic-signal metadata; it is not part of the default --source all federation.

<!-- /licensing -->

<!-- licensing:litvar2 -->
### LitVar2

- BioMCP surfaces: `variant articles <gene> <protein change>`; `variant articles <rsID>`
- Integration mode: `direct_api`
- BioMCP auth: `none`
- Commercial terms: `null`
- Provider access / registration: open public API without a key
- License / terms summary: NCBI/NLM public-domain literature service; variant-literature links are free to query
- Redistribution / reuse summary: query results are broadly reusable, but each paper's own reuse rights stay separate
- Official terms URL: <https://www.ncbi.nlm.nih.gov/research/litvar2-api/>
- Reviewed on: `2026-10-17`
- Notes: LitVar2 joins the variant article search beside PubTator3's annotation route: an autocomplete entry must match the requested gene and protein change, or rsID, exactly, and a LitVar2 failure degrades that source only. `--disable-sources litvar2` turns it off.

<!-- /licensing -->

<!-- licensing:mondo -->
### MONDO

- BioMCP surfaces: `search disease`; `discover <query>`
- Integration mode: `indirect_only`
- BioMCP auth: `not_applicable`
- Commercial terms: `null`
- Via: MyDisease.info
- Provider access / registration: surfaced indirectly through MyDisease.info payloads
- License / terms summary: CC BY 4.0
- Redistribution / reuse summary: reuse is allowed with attribution and ontology version tracking
- Official terms URL: <https://mondo.monarchinitiative.org/pages/download/>
- Reviewed on: `2026-09-27`
- Notes: BioMCP uses MONDO identifiers through MyDisease.info and other aggregators rather than calling MONDO directly.

<!-- /licensing -->

<!-- licensing:pdb -->
### PDB

- BioMCP surfaces: `get protein <id> structures`
- Integration mode: `indirect_only`
- BioMCP auth: `not_applicable`
- Commercial terms: `null`
- Via: UniProt
- Provider access / registration: surfaced through UniProt cross-references; no standalone BioMCP PDB client
- License / terms summary: PDB archive data is CC0 1.0
- Redistribution / reuse summary: data is broadly reusable; attribution to original structure authors is encouraged
- Official terms URL: <https://www.rcsb.org/pages/usage-policy>
- Reviewed on: `2026-09-27`
- Notes: BioMCP currently exposes PDB identifiers from UniProt rather than querying RCSB PDB directly.

<!-- /licensing -->

## Source notes

- `PubMed`, `PubTator3`, `Europe PMC`, `LitSense2`, `NCBI E-utilities`, `PMC OA`, `NCBI ID Converter`, `Semantic Scholar`, and `Figshare` are separate direct inventory rows inside BioMCP's article stack because search, annotation, identifier, PDF metadata, full-text, and asset-byte responsibilities come from different NCBI/EMBL/AI2/Figshare service surfaces. PMC article HTML is documented as a PMC web fallback in the data-source matrix rather than as a separate source-client row.
- `OpenFDA FAERS`, `OpenFDA label`, `OpenFDA shortage`, and `Drugs@FDA` are user-facing provenance labels that resolve back to the `OpenFDA` direct row plus the `Drugs@FDA` indirect row.
- `AlphaFold DB` and `PDB` are indirect-only because BioMCP currently surfaces those structure IDs via `UniProt` cross-references rather than maintaining standalone source clients.
- `COSMIC` is indirect-only provenance through `MyVariant.info`. See its source detail for the supported scope and reuse caveats.
<!-- licensing:cell-model-passports -->
### Cell Model Passports

- BioMCP surfaces: `get cell-line <CVCL_xxxx> xrefs (identifier only)`
- Integration mode: `indirect_only`
- BioMCP auth: `not_applicable`
- Commercial terms: `null`
- Via: cellosaurus
- Provider access / registration: public portal; BioMCP shows Cell Model Passports identifiers resolved through Cellosaurus cross-references and fetches no portal data
- License / terms summary: Sanger DepMap Data Usage Policy: a non-exclusive, non-transferable right to use the data files for internal proprietary research and education; resale of all or a significant part and provision of commercial services are excluded, and commercial API/data use requires prior consent (non-commercial API access needs no registration)
- Redistribution / reuse summary: BioMCP shows the identifier through a Cellosaurus cross-reference only; users who follow it to the portal use the portal under its policy
- Official terms URL: <https://depmap.sanger.ac.uk/documentation/data-usage-policy/>
- Reviewed on: `2026-10-01`
- Notes: Identifier-only entry: BioMCP shows Cell Model Passports identifiers resolved through Cellosaurus cross-references and fetches no portal data.

<!-- /licensing -->

<!-- licensing:depmap -->
### DepMap

- BioMCP surfaces: `get cell-line <CVCL_xxxx> xrefs (identifier only)`
- Integration mode: `indirect_only`
- BioMCP auth: `not_applicable`
- Commercial terms: `null`
- Via: cellosaurus
- Provider access / registration: public portal; BioMCP shows DepMap identifiers resolved through Cellosaurus cross-references and fetches no DepMap data
- License / terms summary: DepMap's own guidance says DepMap-generated data are generally CC BY 4.0 while other hosted projects may use different licenses; each file's Data Usage and Terms must be checked per download, and a 2026-07-28 staff update reports newly generated data carry restricted terms (research use free, commercial use separately licensed)
- Redistribution / reuse summary: BioMCP shows the identifier through a Cellosaurus cross-reference only; no DepMap bytes enter BioMCP output beyond the identifier
- Official terms URL: <https://forum.depmap.org/t/license-for-data-found-in-the-depmap-portal/130>
- Reviewed on: `2026-10-01`
- Notes: Identifier-only entry: BioMCP shows DepMap identifiers resolved through Cellosaurus cross-references and fetches no DepMap data. The 2026-09-27 decision stands: restricted DepMap bytes are never rehosted in this public repository.

<!-- /licensing -->

<!-- licensing:cellosaurus -->
### Cellosaurus

- BioMCP surfaces: `search cell-line <name>`; `get cell-line <id>`; `get cell-line <id> xrefs`; `get cell-line <id> variants`
- Integration mode: `direct_api`
- BioMCP auth: `none`
- Commercial terms: `null`
- Provider access / registration: open public API, no key and no published rate limit
- License / terms summary: CC BY 4.0
- Redistribution / reuse summary: reuse is allowed with attribution and the requested citation on reuse
- Official terms URL: <https://www.cellosaurus.org/description.html>
- Reviewed on: `2026-09-16`
- Notes: Cellosaurus is the identity source for cell lines and the RRID authority. Every BioMCP cell line output names the release, CC BY 4.0, and the citation Bairoch A. J. Biomol. Tech. 29:25-38 (2018). Cellosaurus is the RRID authority for cell lines, and the RRID is `RRID:` plus the primary accession. The requested citation is Bairoch A. "The Cellosaurus, a cell line knowledge resource." J. Biomol. Tech. 29:25-38 (2018).

<!-- /licensing -->

<!-- licensing:chembl -->
### ChEMBL

- BioMCP surfaces: `get cell-line <id> chembl`; `get drug <name> indications`; `get drug <name> targets`
- Integration mode: `direct_api`
- BioMCP auth: `none`
- Commercial terms: `null`
- Provider access / registration: open public API
- License / terms summary: ChEMBL data is provided under Creative Commons Attribution-ShareAlike 3.0 (CC BY-SA 3.0); attribution is required and share-alike applies to adaptations (2026-09-27 pass added the ShareAlike obligation the earlier summary omitted).
- Redistribution / reuse summary: reuse, redistribution and adaptation are allowed with attribution under ShareAlike; preserve ChEMBL IDs and name the release, and note that compound property calculations derived from commercial software carry their own terms
- Official terms URL: <https://www.ebi.ac.uk/chembl/>
- Reviewed on: `2026-09-27`
- Notes: BioMCP only queries live ChEMBL endpoints and does not ship ChEMBL data in the repository. Drug target output keeps ChEMBL in the generic target/mechanism section while CIViC variant labels are rendered separately when present. The about page reads "The ChEMBL data is made available on a Creative Commons Attribution-Share Alike 3.0 Unported License", and the release LICENSE file at https://ftp.ebi.ac.uk/pub/databases/chembl/ChEMBLdb/latest/LICENSE carries the same grant; the shipped ChEMBL attribution line has said CC BY-SA 3.0 since ticket 1214.

<!-- /licensing -->

<!-- licensing:civic -->
### CIViC

- BioMCP surfaces: `get gene <symbol> civic`; `get drug <name> targets`; `get drug <name> civic`; `get disease <id> variants`
- Integration mode: `direct_api`
- BioMCP auth: `none`
- Commercial terms: `null`
- Provider access / registration: open public API
- License / terms summary: open community knowledgebase; CIViC content is published for unrestricted reuse. Verified 2026-09-28 against the live FAQ (plain HTML): CIViC content is released under CC0 1.0 Universal; the source code is MIT; CIViC is intended for purely research purposes.
- Redistribution / reuse summary: reuse is broadly permitted; attribution remains best practice
- Official terms URL: <https://docs.civicdb.org/en/latest/about/faq.html>
- Reviewed on: `2026-09-28`
- Notes: CIViC is treated here as an open-access evidence source surfaced directly by BioMCP. Drug target output may add a separate CIViC variant-target annotation line without merging those labels into the generic targets section.

<!-- /licensing -->

<!-- licensing:clingen -->
### ClinGen

- BioMCP surfaces: `get gene <symbol> clingen`; `variant erepo <CAid>`
- Integration mode: `direct_api`
- BioMCP auth: `none`
- Commercial terms: `null`
- Provider access / registration: public web/API access
- License / terms summary: public ClinGen curation resources with publication and attribution expectations
- Redistribution / reuse summary: generally queryable and reusable, but users should preserve attribution and source context
- Official terms URL: <https://clinicalgenome.org/>
- Reviewed on: `2026-09-27`
- Notes: The gene clingen section reports gene-validity and dosage-sensitivity acquisition independently, preserves partial evidence, and distinguishes a missing dosage classification from a literal ClinGen no-evidence classification. BioMCP uses distinct ClinGen Search and Evidence Repository origins; ERepo assertions preserve panel source facts and versioned SEPIO provenance without interpretation. Open Targets currently lists ClinGen under CC0 for its own ingestion, but BioMCP links to ClinGen's official project site because that is the provider surface users encounter directly.

<!-- /licensing -->

<!-- licensing:clingen_car -->
### ClinGen Allele Registry

- BioMCP surfaces: `variant normalize car <HGVS>`
- Integration mode: `direct_api`
- BioMCP auth: `none`
- Commercial terms: `null`
- Provider access / registration: public web/API access
- License / terms summary: public ClinGen curation resources with publication and attribution expectations
- Redistribution / reuse summary: generally queryable and reusable, but users should preserve attribution and source context
- Official terms URL: <https://reg.genome.network>
- Reviewed on: `2026-07-24`
- Notes: BioMCP performs read-only projected CAR lookups for supported versioned RefSeq HGVS values. BioMCP makes read-only projected CAR lookups and returns source facts without assigning clinical meaning.

<!-- /licensing -->

<!-- licensing:clingen_cspec -->
### ClinGen CSpec

- BioMCP surfaces: `gene cspec <GENE>`; `gene_cspec`
- Integration mode: `direct_api`
- BioMCP auth: `none`
- Commercial terms: `null`
- Provider access / registration: public web/API access
- License / terms summary: public ClinGen curation resources with publication and attribution expectations
- Redistribution / reuse summary: generally queryable and reusable, but users should preserve attribution and source context
- Official terms URL: <https://cspec.clinicalgenome.org>
- Reviewed on: `2026-07-23`
- Notes: BioMCP preserves exact selected CSpec document bytes behind bounded local capture handles and returns source facts without ACMG interpretation.

<!-- /licensing -->

<!-- licensing:clingen_erepo -->
### ClinGen ERepo

- BioMCP surfaces: `variant erepo <CAid>`
- Integration mode: `direct_api`
- BioMCP auth: `none`
- Commercial terms: `null`
- Provider access / registration: public web/API access
- License / terms summary: public ClinGen curation resources with publication and attribution expectations
- Redistribution / reuse summary: generally queryable and reusable, but users should preserve attribution and source context
- Official terms URL: <https://erepo.clinicalgenome.org>
- Reviewed on: `2026-07-23`
- Notes: BioMCP retrieves only bounded exact-CAid summaries and explicitly selected versioned detail documents. BioMCP exposes bounded source facts and explicitly selected versioned detail without interpreting expert assertions.

<!-- /licensing -->

<!-- licensing:clingen_ldh -->
### ClinGen LDH

- BioMCP surfaces: `variant articles --verify-identity`; `variant_articles`
- Integration mode: `direct_api`
- BioMCP auth: `none`
- Commercial terms: `null`
- Provider access / registration: public web/API access
- License / terms summary: public ClinGen curation resources with publication and attribution expectations
- Redistribution / reuse summary: generally queryable and reusable, but users should preserve attribution and source context
- Official terms URL: <https://ldh.genome.network>
- Reviewed on: `2026-07-26`
- Notes: BioMCP uses bounded post-retrieval LDH annotation lookups only as optional article identity observations; it does not discover, remove, rank, or supply negative evidence for candidates.

<!-- /licensing -->

<!-- licensing:clinicaltrials-gov -->
### ClinicalTrials.gov

- BioMCP surfaces: `search trial`; `get trial <nct_id>`
- Integration mode: `direct_api`
- BioMCP auth: `none`
- Commercial terms: `null`
- Provider access / registration: open public API
- License / terms summary: U.S. government public information service
- Redistribution / reuse summary: records are broadly reusable; preserve identifiers and avoid implying NLM endorsement
- Official terms URL: <https://clinicaltrials.gov/data-api/about-api>
- Reviewed on: `2026-09-27`
- Notes: BioMCP uses the public v2 API as the baseline trial backend.

<!-- /licensing -->

<!-- licensing:gencc -->
### GenCC

- BioMCP surfaces: `get gene <symbol> gencc`; `gencc sync`
- Integration mode: `direct_api`
- BioMCP auth: `none`
- Commercial terms: `null`
- Provider access / registration: public weekly new-format CSV export
- License / terms summary: CC0 1.0 data with requested attribution to GenCC and contributing submitters
- Redistribution / reuse summary: reuse is permitted under CC0; preserve GenCC and submitter attribution and the non-diagnostic context
- Official terms URL: <https://thegencc.org/terms>
- Reviewed on: `2026-09-06`
- Notes: BioMCP retains a private normalized local index, refreshes no more than once daily after failure, and exposes separate submission-level assertions without computing consensus. The public export excludes restricted OMIM data.

<!-- /licensing -->

<!-- licensing:complexportal -->
### ComplexPortal

- BioMCP surfaces: `get protein <id> complexes`
- Integration mode: `direct_api`
- BioMCP auth: `none`
- Commercial terms: `null`
- Provider access / registration: open public API
- License / terms summary: EMBL-EBI open data service
- Redistribution / reuse summary: reuse follows EMBL-EBI resource terms and any embedded third-party source obligations
- Official terms URL: <https://www.ebi.ac.uk/complexportal/>
- Reviewed on: `2026-09-27`
- Notes: BioMCP calls the IntAct Complex Portal web service. Complex membership data is queried on demand only.

<!-- /licensing -->

<!-- licensing:cpic -->
### CPIC

- BioMCP surfaces: `search pgx`; `get pgx <gene_or_drug>`
- Integration mode: `direct_api`
- BioMCP auth: `none`
- Commercial terms: `null`
- Provider access / registration: open public API
- License / terms summary: cpicpgx.org now redirects to ClinPGx and its former CC0 licence pages are gone; the successor data-usage policy states ClinPGx data is CC BY-SA 4.0. Whether that licence governs CPIC content is not stated on any live page; treat CPIC data as attribution-plus-share-alike until CPIC publishes terms on its own domain again.
- Redistribution / reuse summary: content reuse is broadly allowed, but the CPIC mark/logo has separate restrictions
- Official terms URL: <https://www.clinpgx.org/page/dataUsagePolicy>
- Reviewed on: `2026-09-27`
- Notes: CPIC announced in March 2026 that its content is moving to ClinPGx; cpicpgx.org now redirects and the old CPIC licence pages 404, so the ClinPGx policy page is the only live terms statement.

<!-- /licensing -->

<!-- licensing:dgidb -->
### DGIdb

- BioMCP surfaces: `get gene <symbol> interactions`; `get drug <name> interactions`
- Integration mode: `direct_api`
- BioMCP auth: `none`
- Commercial terms: `null`
- Provider access / registration: open public API
- License / terms summary: open interaction service; aggregated claims may still reflect upstream source terms
- Redistribution / reuse summary: treat DGIdb as an aggregation layer and preserve source attribution for underlying claim providers
- Official terms URL: <https://www.dgidb.org/about>
- Reviewed on: `2026-09-27`
- Notes: DGIdb itself is open to query, but it aggregates claims from many external drug-gene sources.

<!-- /licensing -->

<!-- licensing:cdc-cvx -->
### CDC CVX/MVX

- BioMCP surfaces: `search drug <name> --region eu|all`; `search drug <name> --region who --product-type vaccine`; `search drug <name> (default plain-name all-region search)`; `biomcp health`; `biomcp cvx sync`
- Integration mode: `direct_api`
- BioMCP auth: `none`
- Commercial terms: `null`
- Provider access / registration: on-demand local download by BioMCP on first EU/default/WHO-vaccine bridge use, or manual preseed via BIOMCP_CVX_DIR
- License / terms summary: most CDC website materials are public domain, but attribution, disclaimer, and exceptions for third-party or non-federal content still apply
- Redistribution / reuse summary: reuse is generally allowed with CDC attribution and non-endorsement language; avoid CDC logos and review exceptions before republishing
- Official terms URL: <https://www.cdc.gov/other/agencymaterials.html>
- Reviewed on: `2026-04-17`
- Notes: BioMCP auto-downloads cvx.txt, TRADENAME.txt, and mvx.txt into BIOMCP_CVX_DIR or the default data directory on first use, refreshes stale files after 30 days, supports explicit refresh via biomcp cvx sync, augments EMA/default vaccine search plus explicit WHO vaccine name/brand search after MyChem misses, and does not use the CVX root for pure --region us searches or WHO finished-pharma/API lookups.

<!-- /licensing -->

<!-- licensing:vaers -->
### CDC WONDER VAERS

- BioMCP surfaces: `search adverse-event --source vaers`; `search adverse-event --source all`; `biomcp health`
- Integration mode: `direct_api`
- BioMCP auth: `none`
- Commercial terms: `null`
- Provider access / registration: open public CDC WONDER web/API access; consent to the CDC WONDER data use restrictions is required
- License / terms summary: CDC WONDER data use restrictions require statistical reporting/analysis use and prohibit re-identification attempts
- Redistribution / reuse summary: reuse is allowed for statistical reporting and analysis with source attribution, but do not try to identify individuals or publish identifying linkages
- Official terms URL: <https://wonder.cdc.gov/datause.html>
- Reviewed on: `2026-04-18`
- Notes: BioMCP calls the CDC WONDER VAERS D8 XML POST endpoint for aggregate vaccine adverse-event summaries only. The source is aggregate-only in this ticket; case-level VAERS export and direct report detail retrieval are intentionally out of scope.

<!-- /licensing -->

<!-- licensing:ema -->
### EMA

- BioMCP surfaces: `search drug --region eu|all`; `get drug <name> regulatory --region eu|all`; `get drug <name> safety --region eu|all`; `get drug <name> shortage --region eu|all`
- Integration mode: `direct_api`
- BioMCP auth: `none`
- Commercial terms: `null`
- Provider access / registration: on-demand local download by BioMCP on first EMA use, or manual preseed via BIOMCP_EMA_DIR
- License / terms summary: EMA website material may be reused with source attribution; third-party content can carry separate rights
- Redistribution / reuse summary: EMA-published website data is generally reusable with attribution, but embedded third-party materials may need separate permission
- Official terms URL: <https://www.ema.europa.eu/en/about-us/about-website/legal-notice>
- Reviewed on: `2026-03-24`
- Notes: BioMCP auto-downloads the EMA human-medicines JSON batch into BIOMCP_EMA_DIR or the default data directory on first use, refreshes stale files, supports explicit refresh via biomcp ema sync, accepts `ema` as an input alias for the canonical `eu` region value, and treats omitted `--region` on `get drug <name> regulatory` as the combined U.S.+EU direct-regulatory path. The download landing page is <https://www.ema.europa.eu/en/about-us/about-website/download-website-data-json-data-format>.

<!-- /licensing -->

<!-- licensing:who-prequalification -->
### WHO Prequalification

- BioMCP surfaces: `search drug --region who|all`; `search drug --indication <disease> --region who`; `search drug <name> --region who --product-type <finished_pharma|api|vaccine>`; `get drug <name> regulatory --region who|all`
- Integration mode: `direct_api`
- BioMCP auth: `none`
- Commercial terms: `null`
- Provider access / registration: on-demand local download by BioMCP on first WHO use, or manual preseed via BIOMCP_WHO_DIR
- License / terms summary: WHO content generally requires attribution; commercial reuse may require permission and third-party materials can carry separate rights
- Redistribution / reuse summary: preserve WHO attribution, review commercial-use conditions, and check third-party material rights before republishing
- Official terms URL: <https://www.who.int/about/policies/publishing/copyright>
- Reviewed on: `2026-04-10`
- Notes: BioMCP auto-downloads the WHO finished-pharmaceutical-products CSV, active-pharmaceutical-ingredients CSV, and vaccine CSV into BIOMCP_WHO_DIR or the default data directory on first use, refreshes stale files, supports WHO-only `--product-type <finished_pharma|api|vaccine>` filtering on explicit `--region who` searches, and supports explicit refresh via biomcp who sync. WHO vaccine support is search-only; `get drug <name> regulatory --region who|all` remains finished-pharma/API only.

<!-- /licensing -->

<!-- licensing:gtr -->
### NCBI Genetic Testing Registry

- BioMCP surfaces: `search diagnostic`; `get diagnostic <gtr_accession> [genes|conditions|methods|regulatory]`; `biomcp health`; `biomcp gtr sync`
- Integration mode: `direct_api`
- BioMCP auth: `none`
- Commercial terms: `null`
- Provider access / registration: public FTP bulk export; BioMCP downloads and reads local copies on demand, or operators can preseed `BIOMCP_GTR_DIR` manually
- License / terms summary: NLM/NCBI government data service with public bulk-export access
- Redistribution / reuse summary: GTR records are broadly queryable and reusable, but preserve accession provenance and review any embedded third-party identifiers separately
- Official terms URL: <https://www.ncbi.nlm.nih.gov/gtr/docs/faq/>
- Reviewed on: `2026-04-17`
- Notes: BioMCP requires `test_version.gz` and `test_condition_gene.txt`, auto-downloads them on first diagnostic use, refreshes stale files after 7 days, and supports explicit refresh via `biomcp gtr sync`. BioMCP auto-downloads `test_version.gz` and `test_condition_gene.txt` into `BIOMCP_GTR_DIR` or the default data directory on first use, refreshes stale files after 7 days, and supports explicit refresh via `biomcp gtr sync`.

<!-- /licensing -->

<!-- licensing:who-ivd -->
### WHO Prequalified IVD

- BioMCP surfaces: `search diagnostic --source who-ivd|all`; `get diagnostic <who_ivd_product_code> [conditions|regulatory]`; `biomcp health`; `biomcp who-ivd sync`
- Integration mode: `direct_api`
- BioMCP auth: `none`
- Commercial terms: `null`
- Provider access / registration: public WHO CSV export; BioMCP downloads and reads a local copy on demand, or operators can preseed `BIOMCP_WHO_IVD_DIR` manually
- License / terms summary: WHO content generally requires attribution; commercial reuse may require permission and third-party materials can carry separate rights
- Redistribution / reuse summary: Preserve WHO attribution, review commercial-use conditions, and verify third-party material rights before republishing WHO IVD-derived content
- Official terms URL: <https://www.who.int/about/policies/publishing/copyright>
- Reviewed on: `2026-04-18`
- Notes: BioMCP requires `who_ivd.csv`, auto-downloads it into `BIOMCP_WHO_IVD_DIR` or the default data directory on first diagnostic WHO IVD use, refreshes stale files after 72 hours, and supports explicit refresh via `biomcp who-ivd sync`.

<!-- /licensing -->

<!-- licensing:europe-pmc -->
### Europe PMC

- BioMCP surfaces: `search article`; `get article <pmid>`; `get article <id> fulltext`; `get article <id> assets`; `get article <id> asset <asset-key>`
- Integration mode: `direct_api`
- BioMCP auth: `none`
- Commercial terms: `null`
- Provider access / registration: open public API
- License / terms summary: open literature metadata service; article and full-text licenses vary by record
- Redistribution / reuse summary: metadata is broadly reusable, but full text and PDFs remain governed by article-level licenses
- Official terms URL: <https://europepmc.org/RestfulWebService>
- Reviewed on: `2026-09-27`
- Notes: BioMCP uses Europe PMC for search, bibliographic metadata, article full-text XML, and the second article-asset rung after PMC OA. Supplementary ZIP bytes do not imply reusable status; a retained PMC OA license keeps PMC OA source attribution. BioMCP reports unknown reuse unless article metadata or a retained PMC OA manifest supplies a license, and retained PMC licenses keep PMC OA source attribution.

<!-- /licensing -->

<!-- licensing:figshare -->
### Figshare

- BioMCP surfaces: `get article <id> assets`; `get article <id> asset <asset-key>`
- Integration mode: `direct_api`
- BioMCP auth: `none`
- Commercial terms: `null`
- Provider access / registration: open public API
- License / terms summary: public repository API; each article/file carries its own license metadata
- Redistribution / reuse summary: reuse downloaded article assets according to the Figshare item license and preserve Figshare/article provenance
- Official terms URL: <https://figshare.com/terms>
- Reviewed on: `2026-06-06`
- Notes: BioMCP uses Figshare only as an article-asset fallback after PMC OA and Europe PMC assets are unavailable and Semantic Scholar discovery points at a supported Figshare/AACR Figshare article URL. BioMCP downloads bytes from the Figshare API download_url and does not parse supplement contents.

<!-- /licensing -->

<!-- licensing:gprofiler -->
### g:Profiler

- BioMCP surfaces: `enrich <GENE1,GENE2,...>`
- Integration mode: `direct_api`
- BioMCP auth: `none`
- Commercial terms: `null`
- Provider access / registration: open public API
- License / terms summary: open enrichment service with provider citation expectations
- Redistribution / reuse summary: results are queryable and reusable, but cite g:Profiler and any underlying databases you depend on
- Official terms URL: <https://biit.cs.ut.ee/gprofiler/help.cgi>
- Reviewed on: `2026-09-27`
- Notes: BioMCP uses g:Profiler only for top-level gene-set enrichment.

<!-- /licensing -->

<!-- licensing:gtex -->
### GTEx

- BioMCP surfaces: `get gene <symbol> expression`
- Integration mode: `direct_api`
- BioMCP auth: `none`
- Commercial terms: `null`
- Provider access / registration: open public portal/API for public-access expression data
- License / terms summary: NIH-hosted public-access expression resource
- Redistribution / reuse summary: public summary/expression views are broadly reusable; controlled-access data remains outside BioMCP's scope
- Official terms URL: <https://gtexportal.org/home/documentationPage>
- Reviewed on: `2026-09-27`
- Notes: BioMCP only queries public GTEx expression endpoints, not controlled-access donor-level data.

<!-- /licensing -->

<!-- licensing:gwas-catalog -->
### GWAS Catalog

- BioMCP surfaces: `search gwas`; `get variant <id> gwas`
- Integration mode: `direct_api`
- BioMCP auth: `none`
- Commercial terms: `null`
- Provider access / registration: open public API
- License / terms summary: EMBL-EBI resource terms; summary statistics may carry separate licenses
- Redistribution / reuse summary: query results are generally reusable, but dataset-level summary statistics can have separate downstream terms
- Official terms URL: <https://www.ebi.ac.uk/gwas/docs/about>
- Reviewed on: `2026-09-27`
- Notes: Open Targets reports GWAS Catalog summary statistics under CC0 while the service remains under EMBL-EBI terms.

<!-- /licensing -->

<!-- licensing:hpo-jax-api -->
### HPO JAX API

- BioMCP surfaces: `search phenotype`; `discover phenotype term resolution`
- Integration mode: `direct_api`
- BioMCP auth: `none`
- Commercial terms: `null`
- Provider access / registration: open public ontology API
- License / terms summary: open HPO data with attribution and integrity requirements
- Redistribution / reuse summary: reuse is allowed, but users should preserve attribution, version context, and source integrity
- Official terms URL: <https://human-phenotype-ontology.github.io/license.html>
- Reviewed on: `2026-09-27`
- Notes: The HPO project's own license page is more specific than generic JAX site text and is the clearest official usage statement currently exposed.

<!-- /licensing -->

<!-- licensing:interpro -->
### InterPro

- BioMCP surfaces: `get protein <id> domains`
- Integration mode: `direct_api`
- BioMCP auth: `none`
- Commercial terms: `null`
- Provider access / registration: open public API
- License / terms summary: EMBL-EBI open data resource
- Redistribution / reuse summary: reuse follows InterPro/EMBL-EBI resource terms and any embedded member-database obligations
- Official terms URL: <https://www.ebi.ac.uk/about/terms-of-use>
- Reviewed on: `2026-09-27`
- Notes: InterPro aggregates signatures from multiple member databases; downstream interpretation should keep that provenance.

<!-- /licensing -->

<!-- licensing:medlineplus -->
### MedlinePlus

- BioMCP surfaces: `discover plain-language topics`
- Integration mode: `direct_api`
- BioMCP auth: `none`
- Commercial terms: `null`
- Provider access / registration: open public search API
- License / terms summary: NLM public-information service with trademark and endorsement guidance
- Redistribution / reuse summary: content is widely reusable, but preserve attribution and avoid implying MedlinePlus/NLM endorsement
- Official terms URL: <https://medlineplus.gov/about/using/>
- Reviewed on: `2026-09-27`
- Notes: BioMCP uses MedlinePlus for best-effort plain-language discover context.

<!-- /licensing -->

<!-- licensing:monarch-initiative -->
### Monarch Initiative

- BioMCP surfaces: `get disease <id> genes`; `get disease <id> models`; `search phenotype`
- Integration mode: `direct_api`
- BioMCP auth: `none`
- Commercial terms: `null`
- Provider access / registration: open public API
- License / terms summary: open integrated knowledge graph; underlying source licenses still matter
- Redistribution / reuse summary: results can be queried openly, but downstream reuse should respect the original sources folded into Monarch
- Official terms URL: <https://monarchinitiative.org/>
- Reviewed on: `2026-09-27`
- Notes: BioMCP uses Monarch as an aggregator for disease, phenotype, and model-organism relationships rather than as the legal source of every embedded assertion.

<!-- /licensing -->

<!-- licensing:mutalyzer -->
### Mutalyzer

- BioMCP surfaces: `variant normalize`
- Integration mode: `direct_api`
- BioMCP auth: `none`
- Commercial terms: `null`
- Provider access / registration: open public API
- License / terms summary: public Mutalyzer normalization service; provider terms apply to returned validation output
- Redistribution / reuse summary: preserve Mutalyzer source identity and returned HGVS/protein fields when reusing results
- Official terms URL: <https://mutalyzer.nl/>
- Reviewed on: `2026-05-22`
- Notes: BioMCP calls GET /normalize/{description} for explicit transcript HGVS and preserves source warnings/status without classifying variants.

<!-- /licensing -->

<!-- licensing:mychem-info -->
### MyChem.info

- BioMCP surfaces: `search drug`; `get drug <name>`
- Integration mode: `direct_api`
- BioMCP auth: `none`
- Commercial terms: `null`
- Provider access / registration: open public API
- License / terms summary: BioThings aggregation service; upstream source terms continue to apply
- Redistribution / reuse summary: do not assume aggregator responses are relicensed; preserve source provenance for downstream reuse. Shared BioThings footer: intended for research purposes; not for emergencies or medical advice (noted 2026-09-27 pass).
- Official terms URL: <https://docs.mychem.info/en/latest/>
- Reviewed on: `2026-09-27`
- Notes: DrugBank and other upstream providers appear in MyChem payloads with their own terms.

<!-- /licensing -->

<!-- licensing:mydisease-info -->
### MyDisease.info

- BioMCP surfaces: `search disease`; `get disease <id>`; `search trial (condition abbreviation note)`; `search trial --source nci (condition grounding)`
- Integration mode: `direct_api`
- BioMCP auth: `none`
- Commercial terms: `null`
- Provider access / registration: open public API
- License / terms summary: BioThings aggregation service; source ontologies and datasets keep their own terms
- Redistribution / reuse summary: treat payloads as aggregated source data rather than a new umbrella license. Shared BioThings footer: intended for research purposes; not for emergencies or medical advice (noted 2026-09-27 pass).
- Official terms URL: <https://docs.mydisease.info/en/latest/>
- Reviewed on: `2026-09-27`
- Notes: Disease Ontology and MONDO appear through MyDisease.info as indirect provenance sources.

<!-- /licensing -->

<!-- licensing:mygene-info -->
### MyGene.info

- BioMCP surfaces: `search gene`; `get gene <symbol>`; `search variant <free-text query>`; `get variant <gene> <change>`; `get article <id>; article entities`
- Integration mode: `direct_api`
- BioMCP auth: `none`
- Commercial terms: `null`
- Provider access / registration: open public API
- License / terms summary: BioThings aggregation service; source-specific terms remain attached to underlying records
- Redistribution / reuse summary: reuse should preserve provenance back to NCBI Gene, UniProt, and other upstream sources. Shared BioThings footer: intended for research purposes; not for emergencies or medical advice (added 2026-09-27 pass).
- Official terms URL: <https://docs.mygene.info/en/latest/>
- Reviewed on: `2026-09-27`
- Notes: BioMCP uses MyGene.info mainly as an identity/normalization layer rather than as the legal origin of all gene data. Free-text variant search confirms a gene-symbol first token, `get variant <gene> <change>` resolves the gene's UniProt accession for the numbering check, and article cards and article entities resolve official symbols for spelled gene names.

<!-- /licensing -->

<!-- licensing:myvariant-info -->
### MyVariant.info

- BioMCP surfaces: `search variant`; `get variant <id>`
- Integration mode: `direct_api`
- BioMCP auth: `none`
- Commercial terms: `null`
- Provider access / registration: open public API
- License / terms summary: BioThings aggregation service; indirect providers retain their own terms
- Redistribution / reuse summary: ClinVar, Cancer Genome Interpreter, and gnomAD-related fields follow their original providers' terms; the COSMIC fields are the v68 pre-licence-change snapshot per MyVariant's own metadata, so the current COSMIC licence does not attach to them. Shared BioThings footer: intended for research purposes; not for emergencies or medical advice (noted 2026-09-27 pass).
- Official terms URL: <https://docs.myvariant.info/en/latest/>
- Reviewed on: `2026-09-27`
- Notes: BioMCP intentionally documents several indirect-only provenance rows that arrive through MyVariant.info payloads. Exact variant-search transcript annotations identify MyVariant.info as the direct carrier and SnpEff `ann` as their indirect annotation provenance.

<!-- /licensing -->

<!-- licensing:ncbi-id-converter -->
### NCBI ID Converter

- BioMCP surfaces: `get article <id> fulltext`
- Integration mode: `direct_api`
- BioMCP auth: `optional_env`
- Commercial terms: `null`
- Credential environment variable: `NCBI_API_KEY`
- Provider access / registration: open public utility; optional My NCBI API key improves throughput
- License / terms summary: NLM public-domain utility service
- Redistribution / reuse summary: utility results are broadly reusable; keep article-level identifiers and downstream article licenses distinct
- Official terms URL: <https://pmc.ncbi.nlm.nih.gov/tools/idconv/>
- API key / account URL: <https://www.ncbi.nlm.nih.gov/account/settings/>
- Reviewed on: `2026-09-27`
- Notes: NCBI API keys increase throughput but are not required for baseline BioMCP usage.

<!-- /licensing -->

<!-- licensing:ncbi-e-utilities -->
### NCBI E-utilities

- BioMCP surfaces: `get article <id> indexing`; `get article <id> fulltext`
- Integration mode: `direct_api`
- BioMCP auth: `optional_env`
- Commercial terms: `null`
- Credential environment variable: `NCBI_API_KEY`
- Provider access / registration: open public E-utilities endpoint; optional My NCBI API key improves throughput
- License / terms summary: NLM public-domain utility service
- Redistribution / reuse summary: utility responses are broadly reusable; article-level full text still follows the returned record's license context
- Official terms URL: <https://www.ncbi.nlm.nih.gov/books/NBK25501/>
- API key / account URL: <https://www.ncbi.nlm.nih.gov/account/settings/>
- Reviewed on: `2026-04-10`
- Notes: BioMCP uses PubMed citation `efetch` for the opt-in indexing section and PMC `efetch` as one XML full-text fallback when Europe PMC does not serve the article; PMC OA and PMC article HTML are separate later fallback surfaces. `NCBI_API_KEY` raises the baseline NCBI E-utilities budget but is not required.

<!-- /licensing -->

<!-- licensing:nih-reporter -->
### NIH Reporter

- BioMCP surfaces: `get disease <name_or_id> funding`; `get gene <symbol> funding`
- Integration mode: `direct_api`
- BioMCP auth: `none`
- Commercial terms: `null`
- Provider access / registration: open public API
- License / terms summary: NIH-operated public funding-reporting service
- Redistribution / reuse summary: project and grant metadata are broadly reusable; preserve NIH Reporter attribution, fiscal-year context, and grant identifiers
- Official terms URL: <https://www.nih.gov/web-policies-notices>
- Reviewed on: `2026-04-11`
- Notes: BioMCP uses the public NIH Reporter v2 projects search API for exact-phrase title/abstract funding lookups over the most recent 5 NIH fiscal years. Funding stays opt-in rather than being included in all.

<!-- /licensing -->

<!-- licensing:ols4 -->
### OLS4

- BioMCP surfaces: `discover <query>`
- Integration mode: `direct_api`
- BioMCP auth: `none`
- Commercial terms: `null`
- Provider access / registration: open public ontology service
- License / terms summary: EMBL-EBI ontology browser; each ontology keeps its own license
- Redistribution / reuse summary: ontology metadata is queryable, but downstream reuse depends on the specific ontology surfaced
- Official terms URL: <https://www.ebi.ac.uk/about/terms-of-use>
- Reviewed on: `2026-09-27`
- Notes: OLS4 is BioMCP's required backbone for discover. It is an ontology index, not a single-license data source.

<!-- /licensing -->

<!-- licensing:fda-orphan-designations -->
### FDA Orphan Drug Designations and Approvals

- BioMCP surfaces: `get drug <name> regulatory --region us|all`
- Integration mode: `direct_api`
- BioMCP auth: `none`
- Commercial terms: `null`
- Provider access / registration: public FDA search service
- License / terms summary: FDA-origin public information
- Redistribution / reuse summary: designation records are broadly reusable; preserve FDA source context and never imply designation is approval
- Official terms URL: <https://www.fda.gov/about-fda/about-website/website-policies>
- Reviewed on: `2026-09-11`
- Notes: BioMCP performs bounded live searches of FDA's Orphan Drug Designations and Approvals table for U.S. regulatory cards. Orphan designation and FDA approval remain separate facts.

<!-- /licensing -->

<!-- licensing:opencitations -->
### OpenCitations

- BioMCP surfaces: `article citation-evidence <citing-id> <cited-id>`
- Integration mode: `direct_api`
- BioMCP auth: `none`
- Commercial terms: `null`
- Provider access / registration: free public REST API without authentication
- License / terms summary: OpenCitations datasets are released under a CC0 public domain dedication; website text is CC BY 4.0 and the software is ISC
- Redistribution / reuse summary: citation data is dedicated to the public domain and may be freely reused for any purpose, including commercial use; the OpenCitations trademark policy still governs the name
- Official terms URL: <https://opencitations.net/>
- Reviewed on: `2026-09-16`
- Notes: BioMCP queries the OpenCitations Index reference endpoint for the citing DOI only when a citation-evidence call would otherwise return no passage, confirming that the directed edge exists. OpenCitations carries no passages, so it never supplies quoted text.

<!-- /licensing -->

<!-- licensing:openfda -->
### OpenFDA

- BioMCP surfaces: `search adverse-event`; `get drug <name> label`; `get drug <name> approvals`; `get diagnostic <diagnostic_id> regulatory`
- Integration mode: `direct_api`
- BioMCP auth: `optional_env`
- Commercial terms: `null`
- Credential environment variable: `OPENFDA_API_KEY`
- Provider access / registration: open public API; optional key increases quota headroom
- License / terms summary: FDA-origin public data and API terms
- Redistribution / reuse summary: data is broadly reusable, but avoid implying FDA endorsement and preserve source context
- Official terms URL: <https://open.fda.gov/apis/authentication/>
- API key / account URL: <https://open.fda.gov/apis/authentication/>
- Reviewed on: `2026-04-18`
- Notes: The authentication page says API keys are required while also publishing no-key quotas. BioMCP documents the real runtime behavior: baseline use works without a key, with higher quotas when one is configured. Diagnostic regulatory uses the OpenFDA device 510(k) and PMA endpoints as an exact-name-first live overlay rather than a mirrored device catalog.

<!-- /licensing -->

<!-- licensing:opentargets -->
### OpenTargets

- BioMCP surfaces: `get gene <symbol> diseases`; `get drug <name> targets`; `get disease <id> genes`
- Integration mode: `direct_api`
- BioMCP auth: `none`
- Commercial terms: `null`
- Provider access / registration: open public GraphQL API
- License / terms summary: Open Targets data is CC0; platform code is Apache 2.0
- Redistribution / reuse summary: platform data is dedicated to the public domain, but linked evidence still carries source provenance
- Official terms URL: <https://platform-docs.opentargets.org/licence>
- Reviewed on: `2026-09-27`
- Notes: The licence page also lists the licensing status of major upstream datasets consumed by Open Targets. Drug target output keeps Open Targets in the generic target section while CIViC variant annotations, when present, are labeled separately.

<!-- /licensing -->

<!-- licensing:pmc-oa -->
### PMC OA

- BioMCP surfaces: `get article <id> fulltext`; `get article <id> assets`; `get article <id> asset <asset-key>`
- Integration mode: `direct_api`
- BioMCP auth: `optional_env`
- Commercial terms: `null`
- Credential environment variable: `NCBI_API_KEY`
- Provider access / registration: open public utility for the PMC Open Access subset
- License / terms summary: open-access subset only; article licenses vary within PMC OA
- Redistribution / reuse summary: full text is reusable only according to each article's specific PMC Open Access license
- Official terms URL: <https://pmc.ncbi.nlm.nih.gov/tools/openftlist/>
- API key / account URL: <https://www.ncbi.nlm.nih.gov/account/settings/>
- Reviewed on: `2026-08-05`
- Notes: BioMCP resolves a PMC OA version-prefix listing through its flat version-prefix S3 metadata object, then retrieves its declared XML/media objects; it does not ship the article corpus. The retired FTP/archive route is not used because NLM removes legacy files in August 2026. When Europe PMC recovers bytes after a PMC OA route failure, retained PMC OA license facts keep PMC OA source attribution. PMC article HTML is a separate derived fallback, and returned full text/assets are still governed by article-level licenses. BioMCP also resolves recognized PMC JATS/HTML supplement links behind stable BioMCP handles without exposing provider URLs.

<!-- /licensing -->

<!-- licensing:pubmed -->
### PubMed

- BioMCP surfaces: `search article`; `get article <id>`; `get article <id> indexing`
- Integration mode: `direct_api`
- BioMCP auth: `optional_env`
- Commercial terms: `null`
- Credential environment variable: `NCBI_API_KEY`
- Provider access / registration: open public search/metadata service; optional My NCBI API key improves throughput
- License / terms summary: NLM public-domain search and metadata service
- Redistribution / reuse summary: search results are broadly reusable, but article-level abstracts, full text, and downstream reuse still depend on the returned record context
- Official terms URL: <https://www.ncbi.nlm.nih.gov/books/NBK25501/>
- API key / account URL: <https://www.ncbi.nlm.nih.gov/account/settings/>
- Reviewed on: `2026-04-10`
- Notes: BioMCP uses PubMed as a first-class article search source and fetches citation XML for opt-in indexing with associated affiliations, ORCID, and structured MeSH. It keeps separate rows for PubTator3 annotations, NCBI E-utilities/PMC OA XML full text, NCBI ID Converter bridging, and Semantic Scholar PDF metadata; PMC article HTML is documented as a PMC web fallback in the data-source matrix.

<!-- /licensing -->

<!-- licensing:pubtator3 -->
### PubTator3

- BioMCP surfaces: `search article`; `get article <pmid> annotations`
- Integration mode: `direct_api`
- BioMCP auth: `optional_env`
- Commercial terms: `null`
- Credential environment variable: `NCBI_API_KEY`
- Provider access / registration: open public API; the provider asks for no more than three requests a second and publishes no keyed rate
- License / terms summary: NCBI/NLM public-domain annotation service
- Redistribution / reuse summary: results are broadly reusable, but preserve PMID/source provenance and article-level rights separately
- Official terms URL: <https://www.ncbi.nlm.nih.gov/research/pubtator3/api>
- API key / account URL: <https://www.ncbi.nlm.nih.gov/account/settings/>
- Reviewed on: `2026-10-16`
- Notes: BioMCP uses PubTator3 for article search fan-out and article annotation, paced at the provider's published three requests a second whether or not `NCBI_API_KEY` is set.

<!-- /licensing -->

<!-- licensing:quickgo -->
### QuickGO

- BioMCP surfaces: `get gene <symbol> go`
- Integration mode: `direct_api`
- BioMCP auth: `none`
- Commercial terms: `null`
- Provider access / registration: open public API
- License / terms summary: GO/EMBL-EBI open data service
- Redistribution / reuse summary: query results are generally reusable; preserve GO/EMBL-EBI attribution where expected
- Official terms URL: <https://www.ebi.ac.uk/about/terms-of-use>
- Reviewed on: `2026-09-27`
- Notes: QuickGO exposes GO data and annotations; some embedded evidence sources can carry their own provenance requirements.

<!-- /licensing -->

<!-- licensing:reactome -->
### Reactome

- BioMCP surfaces: `search pathway`; `get pathway <id>`; `get gene <symbol> pathways`
- Integration mode: `direct_api`
- BioMCP auth: `none`
- Commercial terms: `null`
- Provider access / registration: open public API
- License / terms summary: Reactome database data and derived files are CC0; illustrations, icons, art and branding are CC BY 4.0; software has separate licenses
- Redistribution / reuse summary: database data and derived files may be reused commercially without permission; attribution is encouraged for data and required for CC BY 4.0 art
- Official terms URL: <https://reactome.org/license>
- Reviewed on: `2026-10-18`
- Notes: BioMCP retrieves database data through the ContentService JSON API. License section 1(c) covers database data and derived files under CC0; section 1(a) covers illustrations, icons, art and branding under CC BY 4.0. Section 1(b) licenses software under Apache 2.0, with exceptions for copyleft dependencies and separately licensed FoamTree software.

<!-- /licensing -->

<!-- licensing:seer-explorer -->
### SEER Explorer

- BioMCP surfaces: `get disease <id> survival`
- Integration mode: `direct_api`
- BioMCP auth: `none`
- Commercial terms: `null`
- Provider access / registration: public website endpoints; no BioMCP account or key path
- License / terms summary: U.S. government cancer-statistics website and public explorer
- Redistribution / reuse summary: summary survival statistics are broadly reusable, but users should preserve SEER/NCI attribution and understand the explorer endpoints are undocumented UI routes
- Official terms URL: <https://seer.cancer.gov/about/using-website.html>
- Reviewed on: `2026-04-10`
- Notes: BioMCP uses the public SEER Explorer site catalog and explorer endpoints for cancer survival summaries. The shipped contract treats malformed or mismatched explorer payloads as temporary unavailability rather than trustworthy data.

<!-- /licensing -->

<!-- licensing:string -->
### STRING

- BioMCP surfaces: `get gene <symbol> interactions`; `get protein <id> interactions`
- Integration mode: `direct_api`
- BioMCP auth: `none`
- Commercial terms: `null`
- Provider access / registration: open public API
- License / terms summary: CC BY 4.0
- Redistribution / reuse summary: reuse is allowed with attribution to STRING and the original publication/resource
- Official terms URL: <https://string-db.org/cgi/access?footer_active_subpage=licensing>
- Reviewed on: `2026-09-27`
- Notes: BioMCP queries STRING network endpoints on demand and does not package STRING interaction datasets.

<!-- /licensing -->

<!-- licensing:uniprot -->
### UniProt

- BioMCP surfaces: `get protein <id>`; `get gene <symbol> protein`; `get variant <gene> <change>`
- Integration mode: `direct_api`
- BioMCP auth: `none`
- Commercial terms: `null`
- Provider access / registration: open public API
- License / terms summary: CC BY 4.0
- Redistribution / reuse summary: reuse is allowed with attribution; linked cross-references can have their own terms
- Official terms URL: <https://www.uniprot.org/help/license>
- Reviewed on: `2026-09-27`
- Notes: BioMCP also surfaces UniProt cross-references to PDB and AlphaFold DB rather than mirroring those datasets directly. `get variant <gene> <change>` reads the canonical (MANE Select) protein sequence to check a requested reference residue.

<!-- /licensing -->

<!-- licensing:variantvalidator -->
### VariantValidator

- BioMCP surfaces: `variant normalize`
- Integration mode: `direct_api`
- BioMCP auth: `none`
- Commercial terms: `null`
- Provider access / registration: open public API
- License / terms summary: public VariantValidator service; provider terms apply to returned validation output
- Redistribution / reuse summary: preserve VariantValidator source identity, warnings, and genomic descriptions when reusing results
- Official terms URL: <https://variantvalidator.org/>
- Reviewed on: `2026-05-22`
- Notes: BioMCP calls /VariantValidator/variantvalidator/{genome_build}/{variant_description}/{select_transcripts} for explicit transcript HGVS and preserves TranscriptVersionWarning messages without choosing transcripts.

<!-- /licensing -->

<!-- licensing:wikipathways -->
### WikiPathways

- BioMCP surfaces: `search pathway`; `get pathway <id>`
- Integration mode: `direct_api`
- BioMCP auth: `none`
- Commercial terms: `null`
- Provider access / registration: open public API
- License / terms summary: Verified 2026-09-28 against the live terms page: WikiPathways adopts the Creative Commons CC0 waiver for its content (the page says so in plain text). The earlier unverifiable call reflected the retired classic site; the current site's terms page is live.
- Redistribution / reuse summary: pathway content is dedicated to the public domain; attribution is still good scholarly practice
- Official terms URL: <https://www.wikipathways.org/terms.html>
- Reviewed on: `2026-09-28`
- Notes: Verified against the live terms page 2026-09-28: WikiPathways adopts the CC0 waiver for its content; the classic site is retired.

<!-- /licensing -->

## Tier 2 - Credential, account, or license required for the BioMCP feature

<!-- licensing:alphagenome -->
### AlphaGenome

- BioMCP surfaces: `get variant <id> predict`
- Integration mode: `direct_api`
- BioMCP auth: `required_env`
- Commercial terms: `non-commercial`
- Credential environment variable: `ALPHAGENOME_API_KEY`
- Provider access / registration: provider-controlled access to the AlphaGenome service
- License / terms summary: restricted: the hosted AlphaGenome API permits non-commercial use under provider terms
- Redistribution / reuse summary: hosted API outputs are restricted to non-commercial research; provider exceptions for downloadable artifacts and separate cloud commercial access do not grant this API integration those rights
- Official terms URL: <https://google-deepmind-alphagenome.readthedocs-hosted.com/faqs.html#what-terms-of-use-apply-to-alphagenome-outputs>
- API key / account URL: <https://deepmind.google/science/alphagenome/>
- Reviewed on: `2026-10-18`
- Notes: BioMCP calls the hosted gRPC API, not the separately licensed cloud commercial route. The official FAQ restricts API outputs and disallows clinical decision-making.

<!-- /licensing -->

<!-- licensing:disgenet -->
### DisGeNET

- BioMCP surfaces: `get gene <symbol> disgenet`; `get disease <id> disgenet`
- Integration mode: `direct_api`
- BioMCP auth: `required_env`
- Commercial terms: `license-required`
- Credential environment variable: `DISGENET_API_KEY`
- Provider access / registration: account registration and API key required
- License / terms summary: restricted: commercial use requires a commercial license
- Redistribution / reuse summary: standard licenses prohibit redistribution and resale of all or part of the database; custom permission is required
- Official terms URL: <https://support.disgenet.com/support/solutions/articles/202000087487-do-i-need-a-commercial-license-can-i-use-disgenet-data-in-my-product->
- API key / account URL: <https://www.disgenet.com/>
- Reviewed on: `2026-10-18`
- Notes: The provider licensing FAQ requires a commercial license for commercial companies and commercial product research or services. BioMCP does not obtain licenses or redistribution permission.

<!-- /licensing -->

<!-- licensing:nci-cts -->
### NCI CTS

- BioMCP surfaces: `search trial --source nci`; `get trial <nct_id> --source nci`
- Integration mode: `direct_api`
- BioMCP auth: `required_env`
- Commercial terms: `null`
- Credential environment variable: `NCI_API_KEY`
- Provider access / registration: API key required
- License / terms summary: custom provider API terms for the NCI Clinical Trials Search API
- Redistribution / reuse summary: query output is usable for search and review, but downstream reuse should follow NCI API terms and record provenance
- Official terms URL: <https://clinicaltrialsapi.cancer.gov/>
- API key / account URL: <https://clinicaltrialsapi.cancer.gov/>
- Reviewed on: `2026-09-27`
- Notes: BioMCP treats NCI CTS as an alternate oncology-focused trial backend, not the default public trial source.

<!-- /licensing -->

<!-- licensing:orcid -->
### ORCID

- BioMCP surfaces: `get author orcid:<id>`; `author papers orcid:<id>`
- Integration mode: `direct_api`
- BioMCP auth: `required_env`
- Commercial terms: `non-commercial`
- Credential environment variable: `ORCID_ACCESS_TOKEN`
- Provider access / registration: public-read bearer token obtained from ORCID for the /read-public scope
- License / terms summary: restricted: ORCID Public and Anonymous APIs permit non-commercial use; section 2 excludes use in revenue-generating products or services even though public record data is CC0
- Redistribution / reuse summary: display public record data with attribution; do not assume all record data or downstream works are identically licensed
- Official terms URL: <https://info.orcid.org/public-client-terms-of-service/>
- API key / account URL: <https://orcid.org/signin>
- Reviewed on: `2026-10-18`
- Notes: exact public person and works records only; no name search, member API, private fields, or cross-provider identity merge. This supersedes the earlier citation-supplied-evidence-only boundary recorded in 2026-09-07 planning; the ADR lives at `sdlc/planning/adr/0001-call-the-orcid-public-api-for-exact-author-records.md`.

<!-- /licensing -->

<!-- licensing:oncokb -->
### OncoKB

- BioMCP surfaces: `variant oncokb <id>`
- Integration mode: `direct_api`
- BioMCP auth: `required_env`
- Commercial terms: `license-required`
- Credential environment variable: `ONCOKB_TOKEN`
- Provider access / registration: registration and provider-controlled API access
- License / terms summary: custom provider terms; academic research access is no-fee but licensed, commercial/clinical use requires a paid license
- Redistribution / reuse summary: do not assume open redistribution rights for OncoKB data or proprietary treatment descriptions
- Official terms URL: <https://faq.oncokb.org/licensing>
- API key / account URL: <https://www.oncokb.org/account/register>
- Reviewed on: `2026-09-27`
- Notes: The FAQ states that programmatic access for academic use still requires registration and provider approval.

<!-- /licensing -->

<!-- licensing:semantic-scholar -->
### Semantic Scholar

- BioMCP surfaces: `search author`; `get author <semanticscholar:id>`; `search article --source semanticscholar`; `get article <id> tldr`; `get article <id> fulltext --pdf`; `article citations <id>`; `article references <id>`; `article recommendations <id>`
- Integration mode: `direct_api`
- BioMCP auth: `optional_env`
- Commercial terms: `null`
- Credential environment variable: `S2_API_KEY`
- Provider access / registration: Unauthenticated access is available on the shared pool; authenticated access uses the provider-issued API key and remains governed by the API license agreement
- License / terms summary: custom API license agreement
- Redistribution / reuse summary: the API license restricts repackaging, resale, and broad commercial redistribution without expanded licensing
- Official terms URL: <https://www.semanticscholar.org/product/api/license>
- API key / account URL: <https://www.semanticscholar.org/product/api>
- Reviewed on: `2026-09-27`
- Notes: BioMCP can call Semantic Scholar without `S2_API_KEY`, but uses a more conservative shared-pool rate limit and recommends the key for dedicated quota and reliability. Author search/detail expose exact provider records without cross-provider resolution. Article search can use Semantic Scholar through compatible default federation or `--source semanticscholar`. For `get article <id> fulltext --pdf`, BioMCP uses Semantic Scholar `openAccessPdf` metadata, then fetches the PDF after explicit PDF opt-in only when its HTTPS origin is on BioMCP's Semantic Scholar/CDN allowlist; the PDF's article-level reuse terms remain separate.

<!-- /licensing -->

<!-- licensing:umls -->
### UMLS

- BioMCP surfaces: `discover <query> crosswalk enrichment`
- Integration mode: `direct_api`
- BioMCP auth: `required_env`
- Commercial terms: `null`
- Credential environment variable: `UMLS_API_KEY`
- Provider access / registration: UTS account plus acceptance of the UMLS Metathesaurus license
- License / terms summary: custom UMLS Metathesaurus license and terminology-specific appendices
- Redistribution / reuse summary: do not assume unrestricted redistribution; some embedded vocabularies add their own restrictions or affiliate licenses
- Official terms URL: <https://www.nlm.nih.gov/databases/umls.html>
- API key / account URL: <https://uts.nlm.nih.gov/uts/signup-login>
- Reviewed on: `2026-09-27`
- Notes: The UMLS landing page explicitly states that you must accept the license and create a UTS account for access.

<!-- /licensing -->

## Tier 3 - Open or queryable, but with notable terms

<!-- licensing:enrichr -->
### Enrichr

- BioMCP surfaces: `get gene <symbol> ontology`
- Integration mode: `direct_api`
- BioMCP auth: `none`
- Commercial terms: `license-required`
- Provider access / registration: open public API
- License / terms summary: Verified 2026-09-29 against the live terms page: free for academic and non-profit use; commercial use requires a license from Mount Sinai Innovation Partners. The terms state Enrichr is not to be used for treating or diagnosing human subjects.
- Redistribution / reuse summary: Commercial use requires a license from Mount Sinai Innovation Partners; not for treating or diagnosing human subjects. Users are responsible for their own licenses.
- Official terms URL: <https://maayanlab.cloud/Enrichr/templates/help/terms-submenu.html>
- Reviewed on: `2026-09-29`
- Notes: Gene enrichment sections inside BioMCP use Enrichr; top-level `biomcp enrich` uses g:Profiler instead. Academic and non-profit reuse with citation to Enrichr and its libraries; commercial reuse needs the Mount Sinai license; the service must not be used to treat or diagnose human subjects.

<!-- /licensing -->

<!-- licensing:cancerhotspots -->
### Cancerhotspots.org

- BioMCP surfaces: `get variant <gene> <change> all`
- Integration mode: `direct_api`
- BioMCP auth: `none`
- Commercial terms: `null`
- Provider access / registration: public API access
- License / terms summary: public cancer hotspot recurrence resource with provider-specific reuse expectations
- Redistribution / reuse summary: BioMCP queries recurrence counts live and does not redistribute a local cancerhotspots dataset
- Official terms URL: <https://www.cancerhotspots.org/>
- Reviewed on: `2026-06-10`
- Notes: BioMCP uses the byGene hotspot endpoint to expose residue-level tumorCount and exact variantAminoAcid counts for variant detail recurrence provenance.

<!-- /licensing -->

<!-- licensing:cbioportal -->
### cBioPortal

- BioMCP surfaces: `get variant <id> cohort`; `study download`; `study query`
- Integration mode: `direct_api`
- BioMCP auth: `none`
- Commercial terms: `null`
- Provider access / registration: public API access for open studies
- License / terms summary: public API with study-specific downstream terms
- Redistribution / reuse summary: reuse depends on the specific study or consortium behind each dataset
- Official terms URL: <https://www.cbioportal.org/>
- Reviewed on: `2026-09-27`
- Notes: BioMCP uses the public API. TCGA-derived studies are broadly open, while some cohorts such as AACR Project GENIE carry additional access or reuse conditions.

<!-- /licensing -->

<!-- licensing:ddinter -->
### DDInter

- BioMCP surfaces: `biomcp drug interactions <name>`; `get drug <name> interactions`; `biomcp health`; `biomcp ddinter sync`
- Integration mode: `direct_api`
- BioMCP auth: `none`
- Commercial terms: `non-commercial`
- Provider access / registration: public DDInter download bundle; operators run biomcp ddinter sync explicitly or preseed BIOMCP_DDINTER_DIR manually, and normal reads stay local
- License / terms summary: CC BY-NC-SA 4.0 with an explicit completeness disclaimer; absence from the database does not prove no interaction exists
- Redistribution / reuse summary: reuse requires attribution, non-commercial use, and ShareAlike treatment; do not turn missing rows into safety claims or assume commercial redistribution rights
- Official terms URL: <https://ddinter.scbdd.com/terms/>
- Reviewed on: `2026-04-25`
- Notes: BioMCP requires all eight DDInter CSV files. Interaction reads use the installed bundle without maintenance and report fresh/stale state. biomcp ddinter sync explicitly downloads, validates, and publishes a complete replacement bundle; empty results remain scoped to the current DDInter bundle. BioMCP reads the eight required DDInter CSV files from `BIOMCP_DDINTER_DIR` or the default data directory without automatic maintenance.

<!-- /licensing -->

<!-- licensing:gnomad -->
### gnomAD

- BioMCP surfaces: `get gene <symbol> constraint`; `get variant <id> population`
- Integration mode: `direct_api`
- BioMCP auth: `none`
- Commercial terms: `null`
- Provider access / registration: open public browser/API
- License / terms summary: Broad Institute data policies with attribution and service-specific conditions
- Redistribution / reuse summary: querying is open, but users should review the gnomAD policies before bulk reuse or republishing
- Official terms URL: <https://gnomad.broadinstitute.org/policies>
- Reviewed on: `2026-09-27`
- Notes: Open Targets currently reports gnomAD as CC0 for its own ingestion pipeline. BioMCP links to gnomAD's own policy page for user-facing guidance.

<!-- /licensing -->

<!-- licensing:human-protein-atlas -->
### Human Protein Atlas

- BioMCP surfaces: `get gene <symbol> hpa`; `gene cell-lines <symbol> --group <group>`
- Integration mode: `direct_api`
- BioMCP auth: `none`
- Commercial terms: `null`
- Provider access / registration: open public XML and web resources
- License / terms summary: CC BY 4.0 for copyrightable parts of the database
- Redistribution / reuse summary: reuse is allowed with attribution; third-party components may impose extra conditions
- Official terms URL: <https://www.proteinatlas.org/about/licence>
- Reviewed on: `2026-09-18`
- Notes: The terms page reads "The Human Protein Atlas is licensed under the Creative Commons Attribution 4.0 International License for all copyrightable parts of our database." The earlier CC BY-SA 4.0 reading was one of the external sources listed further down that page. The page also requires clear citation for images and specific gene/data pages.

<!-- /licensing -->

<!-- licensing:kegg -->
### KEGG

- BioMCP surfaces: `search pathway`; `get pathway <id>`
- Integration mode: `direct_api`
- BioMCP auth: `none`
- Commercial terms: `license-required`
- Provider access / registration: open website/API for academic use; commercial use requires a license
- License / terms summary: custom KEGG terms; academic users may freely use the website, non-academic use requires a commercial license
- Redistribution / reuse summary: do not assume commercial redistribution rights; query access does not grant a redistribution license
- Official terms URL: <https://www.kegg.jp/kegg/legal.html>
- Reviewed on: `2026-09-27`
- Notes: KEGG's official legal page was updated on October 1, 2024 and explicitly distinguishes academic from non-academic use.

<!-- /licensing -->

<!-- licensing:pharmacodb -->
### PharmacoDB

- BioMCP surfaces: `get cell-line <accession> drug_response`; `get drug <name> cell_lines`; `cell-line drug-response <accession> --dataset <name>`; `drug cell-lines <name> --cell-line <id>`; `drug cell-lines <name> --dataset <name>`
- Integration mode: `direct_api`
- BioMCP auth: `none`
- Commercial terms: `non-commercial`
- Provider access / registration: open public GraphQL endpoint, no key
- License / terms summary: PharmacoDB publishes no licence or terms page; its source code is GPL-3.0 and the describing paper is CC BY-NC 4.0, and the terms for the data itself are unstated by the provider
- Redistribution / reuse summary: treat reuse as non-commercial and attribute PharmacoDB; the provider states no terms for the data, so redistribution rights are not established
- Official terms URL: none published
- Reviewed on: `2026-09-18`
- Notes: Verified 2026-09-18: pharmacodb.ca serves the same 2,258-byte single-page shell on every path (/about, /documentation, /terms, /api) and its application bundle contains no licence string, so the provider publishes no licence or terms page. The code licence is GPL-3.0 at https://github.com/bhklab/PharmacoDB/blob/master/LICENSE. The CC BY-NC 4.0 reading traces to the describing paper, Feizi N, et al. PharmacoDB 2.0. Nucleic Acids Research 2022;50(D1):D1348-D1357, doi:10.1093/nar/gkab1084, which states "This is an Open Access article distributed under the terms of the Creative Commons Attribution-NonCommercial License"; that covers the article, not the database. PharmacoDB publishes no version or release date, so every output carries the retrieval time. Every BioMCP output names the non-commercial term because a user cannot read it from the numbers.

<!-- /licensing -->

<!-- licensing:pharmgkb -->
### PharmGKB

- BioMCP surfaces: `get pgx <gene_or_drug> annotations`
- Integration mode: `direct_api`
- BioMCP auth: `none`
- Commercial terms: `null`
- Provider access / registration: open public API
- License / terms summary: PharmGKB's own domain now redirects to ClinPGx and the former terms pages are gone; the successor data-usage policy (read from the live page's JS bundle) states ClinPGx data is CC BY-SA 4.0. Verified 2026-09-28 against the same page CPIC was, since both routes lead to one policy.
- Redistribution / reuse summary: reuse is allowed with attribution and ShareAlike; some underlying annotations and external assets may add extra constraints
- Official terms URL: <https://www.clinpgx.org/page/dataUsagePolicy>
- Reviewed on: `2026-09-28`
- Notes: PharmGKB has been transitioning to ClinPGx-branded API documentation. BioMCP keeps the public `PharmGKB` source label because that is the domain vocabulary users recognize.

<!-- /licensing -->

## Indirect-only providers surfaced through aggregators

<!-- licensing:cancer-genome-interpreter -->
### Cancer Genome Interpreter

- BioMCP surfaces: `get variant <id>`
- Integration mode: `indirect_only`
- BioMCP auth: `not_applicable`
- Commercial terms: `restricted`
- Via: MyVariant.info
- Provider access / registration: surfaced indirectly through MyVariant.info payloads
- License / terms summary: restricted: custom tool terms restrict external and commercial use
- Redistribution / reuse summary: do not assume commercial reuse rights; the official terms restrict some external and commercial use
- Official terms URL: <https://www.cancergenomeinterpreter.org/conditions>
- Reviewed on: `2026-09-27`
- Notes: There is no standalone CGI source client in BioMCP; provenance appears only when MyVariant includes CGI fields.

<!-- /licensing -->

<!-- licensing:cosmic -->
### COSMIC

- BioMCP surfaces: `get variant <id> cosmic`
- Integration mode: `indirect_only`
- BioMCP auth: `not_applicable`
- Commercial terms: `null`
- Via: MyVariant.info
- Provider access / registration: surfaced indirectly through cached MyVariant.info fields; no standalone BioMCP COSMIC client
- License / terms summary: BioMCP reads COSMIC only through MyVariant.info, whose COSMIC fields are the v68 snapshot — per MyVariant's metadata: 'COSMIC v68 was imported from UCSC database dump. This is the last freely available somatic variants from COSMIC before their licence change' (checked 2026-09-27). The v68-era terms, not the current cosmickb.org licence, govern those fields. The current licence requires a commercial licence for commercial use including patient services and clinical reporting; it governs COSMIC's current direct offering only. The practical limit of the v68 fields is age, not licence.
- Redistribution / reuse summary: MyVariant's COSMIC v68 snapshot predates the current COSMIC licence; treat the fields under the v68-era terms and cite MyVariant as the serving source. Direct use of current COSMIC releases remains commercially restricted.
- Official terms URL: <https://docs.myvariant.info/en/latest/doc/data_source.html>
- Reviewed on: `2026-09-27`
- Notes: This is the most important indirect-only caution row. BioMCP intentionally does not support direct COSMIC querying because of licensing risk. Direct COSMIC integration is not part of BioMCP's supported source surface because the provider's licensing model creates unacceptable redistribution and deployment risk for an MIT-licensed open tool.

<!-- /licensing -->

<!-- licensing:drugbank -->
### DrugBank

- BioMCP surfaces: `get drug <name> interactions`; `get drug <name> label`
- Integration mode: `indirect_only`
- BioMCP auth: `not_applicable`
- Commercial terms: `license-required`
- Via: MyChem.info
- Provider access / registration: surfaced indirectly through MyChem.info payloads
- License / terms summary: custom DrugBank terms of use and licensing
- Redistribution / reuse summary: use or redistribution of DrugBank content requires a DrugBank license; do not assume open downstream rights
- Official terms URL: <https://trust.drugbank.com/drugbank-trust-center/drugbank-terms-of-service>
- Reviewed on: `2026-09-27`
- Notes: DrugBank does not have a standalone BioMCP source client. It appears as provenance carried through MyChem.info.

<!-- /licensing -->

