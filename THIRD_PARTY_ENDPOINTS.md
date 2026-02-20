# Third-Party Endpoints Used by BioMCP

_This file is auto-generated from the endpoint registry._

## Overview

BioMCP connects to 22 external domains across 46 endpoints.

## Endpoints by Category

### Biomedical Literature

#### biorxiv_api

- **URL**: `https://api.biorxiv.org/details/biorxiv`
- **Description**: bioRxiv API for searching biology preprints
- **Data Types**: research_articles
- **Rate Limit**: Not specified
- **Compliance Notes**: Public preprint server, no PII transmitted

#### europe_pmc

- **URL**: `https://www.ebi.ac.uk/europepmc/webservices/rest/search`
- **Description**: Europe PMC REST API for searching biomedical literature
- **Data Types**: research_articles
- **Rate Limit**: Not specified
- **Compliance Notes**: Public EMBL-EBI service, no PII transmitted

#### medrxiv_api

- **URL**: `https://api.biorxiv.org/details/medrxiv`
- **Description**: medRxiv API for searching medical preprints
- **Data Types**: research_articles
- **Rate Limit**: Not specified
- **Compliance Notes**: Public preprint server, no PII transmitted

#### pubtator3_autocomplete

- **URL**: `https://www.ncbi.nlm.nih.gov/research/pubtator3-api/entity/autocomplete/`
- **Description**: PubTator3 API for entity name autocomplete suggestions
- **Data Types**: gene_annotations
- **Rate Limit**: 20 requests/second
- **Compliance Notes**: Public NIH/NCBI service, no PII transmitted

#### pubtator3_export

- **URL**: `https://www.ncbi.nlm.nih.gov/research/pubtator3-api/publications/export/biocjson`
- **Description**: PubTator3 API for fetching full article annotations in BioC-JSON format
- **Data Types**: research_articles
- **Rate Limit**: 20 requests/second
- **Compliance Notes**: Public NIH/NCBI service, no PII transmitted

#### pubtator3_search

- **URL**: `https://www.ncbi.nlm.nih.gov/research/pubtator3-api/search/`
- **Description**: PubTator3 API for searching biomedical literature with entity annotations
- **Data Types**: research_articles
- **Rate Limit**: 20 requests/second
- **Compliance Notes**: Public NIH/NCBI service, no PII transmitted

### Clinical Trials

#### clinicaltrials_search

- **URL**: `https://clinicaltrials.gov/api/v2/studies`
- **Description**: ClinicalTrials.gov API v2 for searching clinical trials
- **Data Types**: clinical_trial_data
- **Rate Limit**: 10 requests/second
- **Compliance Notes**: Public NIH service, may contain trial participant criteria

#### nci_biomarkers

- **URL**: `https://clinicaltrialsapi.cancer.gov/api/v2/biomarkers`
- **Description**: NCI API for biomarkers used in clinical trials
- **Data Types**: clinical_trial_data
- **Rate Limit**: Not specified
- **Authentication**: Optional NCI_API_KEY for increased access
- **Compliance Notes**: Public NCI service, biomarker metadata

#### nci_diseases

- **URL**: `https://clinicaltrialsapi.cancer.gov/api/v2/diseases`
- **Description**: NCI API for cancer disease vocabulary
- **Data Types**: clinical_trial_data
- **Rate Limit**: Not specified
- **Authentication**: Optional NCI_API_KEY for increased access
- **Compliance Notes**: Public NCI service, disease ontology

#### nci_interventions

- **URL**: `https://clinicaltrialsapi.cancer.gov/api/v2/interventions`
- **Description**: NCI API for cancer treatment interventions
- **Data Types**: clinical_trial_data
- **Rate Limit**: Not specified
- **Authentication**: Optional NCI_API_KEY for increased access
- **Compliance Notes**: Public NCI service, intervention metadata

#### nci_organizations

- **URL**: `https://clinicaltrialsapi.cancer.gov/api/v2/organizations`
- **Description**: NCI API for cancer research organizations
- **Data Types**: clinical_trial_data
- **Rate Limit**: Not specified
- **Authentication**: Optional NCI_API_KEY for increased access
- **Compliance Notes**: Public NCI service, organization metadata

#### nci_trials

- **URL**: `https://clinicaltrialsapi.cancer.gov/api/v2/trials`
- **Description**: NCI Clinical Trials Search API for cancer trials
- **Data Types**: clinical_trial_data
- **Rate Limit**: Not specified
- **Authentication**: Optional NCI_API_KEY for increased access
- **Compliance Notes**: Public NCI service, cancer trial data

### Variant Databases

#### ensembl_variation

- **URL**: `https://rest.ensembl.org/variation/human`
- **Description**: Ensembl REST API for human genetic variation data
- **Data Types**: genetic_variants
- **Rate Limit**: 15 requests/second
- **Compliance Notes**: Public EMBL-EBI service, population genetics data

#### gdc_ssm_occurrences

- **URL**: `https://api.gdc.cancer.gov/ssm_occurrences`
- **Description**: NCI GDC API for mutation occurrences in cancer samples
- **Data Types**: cancer_mutations
- **Rate Limit**: Not specified
- **Compliance Notes**: Public NCI service, aggregate cancer genomics data

#### gdc_ssms

- **URL**: `https://api.gdc.cancer.gov/ssms`
- **Description**: NCI GDC API for somatic mutations
- **Data Types**: cancer_mutations
- **Rate Limit**: Not specified
- **Compliance Notes**: Public NCI service, aggregate cancer genomics data

#### mychem_chem

- **URL**: `https://mychem.info/v1/chem`
- **Description**: MyChem.info API for fetching specific drug/chemical details
- **Data Types**: gene_annotations
- **Rate Limit**: 10 requests/second
- **Compliance Notes**: Public BioThings service, drug/chemical annotation data

#### mychem_query

- **URL**: `https://mychem.info/v1/query`
- **Description**: MyChem.info API for querying drug/chemical information
- **Data Types**: gene_annotations
- **Rate Limit**: 10 requests/second
- **Compliance Notes**: Public BioThings service, drug/chemical annotation data

#### mydisease_disease

- **URL**: `https://mydisease.info/v1/disease`
- **Description**: MyDisease.info API for fetching specific disease details
- **Data Types**: gene_annotations
- **Rate Limit**: 10 requests/second
- **Compliance Notes**: Public BioThings service, disease ontology data

#### mydisease_query

- **URL**: `https://mydisease.info/v1/query`
- **Description**: MyDisease.info API for querying disease information
- **Data Types**: gene_annotations
- **Rate Limit**: 10 requests/second
- **Compliance Notes**: Public BioThings service, disease ontology data

#### mygene_gene

- **URL**: `https://mygene.info/v3/gene`
- **Description**: MyGene.info API for fetching specific gene details
- **Data Types**: gene_annotations
- **Rate Limit**: 10 requests/second
- **Compliance Notes**: Public BioThings service, gene annotation data

#### mygene_query

- **URL**: `https://mygene.info/v3/query`
- **Description**: MyGene.info API for querying gene information
- **Data Types**: gene_annotations
- **Rate Limit**: 10 requests/second
- **Compliance Notes**: Public BioThings service, gene annotation data

#### myvariant_query

- **URL**: `https://myvariant.info/v1/query`
- **Description**: MyVariant.info API for querying genetic variants
- **Data Types**: genetic_variants
- **Rate Limit**: 1000 requests/hour (anonymous)
- **Compliance Notes**: Public service aggregating variant databases, no patient data

#### myvariant_variant

- **URL**: `https://myvariant.info/v1/variant`
- **Description**: MyVariant.info API for fetching specific variant details
- **Data Types**: genetic_variants
- **Rate Limit**: 1000 requests/hour (anonymous)
- **Compliance Notes**: Public service aggregating variant databases, no patient data

### Cancer Genomics

#### cbioportal_api

- **URL**: `https://www.cbioportal.org/api`
- **Description**: cBioPortal API for cancer genomics data
- **Data Types**: cancer_mutations, clinical_trial_data
- **Rate Limit**: 5 requests/second
- **Authentication**: Optional API token for increased rate limits
- **Compliance Notes**: Public MSKCC/Dana-Farber service, aggregate cancer genomics

#### cbioportal_cancer_types

- **URL**: `https://www.cbioportal.org/api/cancer-types`
- **Description**: cBioPortal API for cancer type hierarchy
- **Data Types**: cancer_mutations
- **Rate Limit**: 5 requests/second
- **Compliance Notes**: Public MSKCC/Dana-Farber service, cancer type metadata

#### cbioportal_genes

- **URL**: `https://www.cbioportal.org/api/genes`
- **Description**: cBioPortal API for gene information
- **Data Types**: gene_annotations
- **Rate Limit**: 5 requests/second
- **Compliance Notes**: Public MSKCC/Dana-Farber service, gene metadata

#### cbioportal_molecular_profiles

- **URL**: `https://www.cbioportal.org/api/molecular-profiles`
- **Description**: cBioPortal API for molecular profiles
- **Data Types**: cancer_mutations
- **Rate Limit**: 5 requests/second
- **Compliance Notes**: Public MSKCC/Dana-Farber service, study metadata

#### cbioportal_mutations

- **URL**: `https://www.cbioportal.org/api/mutations`
- **Description**: cBioPortal API for mutation data
- **Data Types**: cancer_mutations
- **Rate Limit**: 5 requests/second
- **Compliance Notes**: Public MSKCC/Dana-Farber service, aggregate mutation data

#### cbioportal_studies

- **URL**: `https://www.cbioportal.org/api/studies`
- **Description**: cBioPortal API for cancer studies
- **Data Types**: clinical_trial_data, cancer_mutations
- **Rate Limit**: 5 requests/second
- **Compliance Notes**: Public MSKCC/Dana-Farber service, study metadata

#### oncokb_curated_genes

- **URL**: `https://demo.oncokb.org/api/v1/utils/allCuratedGenes`
- **Description**: OncoKB demo API for retrieving curated cancer genes (BRAF, ROS1, TP53)
- **Data Types**: gene_annotations
- **Rate Limit**: Not specified
- **Authentication**: None (demo server). Set ONCOKB_TOKEN for production access.
- **Compliance Notes**: Public MSK OncoKB demo service, no authentication required. Production (www.oncokb.org) requires ONCOKB_TOKEN.

#### oncokb_gene_annotation

- **URL**: `https://demo.oncokb.org/api/v1/genes/{gene}`
- **Description**: OncoKB demo API for gene-level annotations and therapeutic implications
- **Data Types**: gene_annotations
- **Rate Limit**: Not specified
- **Authentication**: None (demo server). Set ONCOKB_TOKEN for production access.
- **Compliance Notes**: Public MSK OncoKB demo service, limited to 3 genes. Production (www.oncokb.org) requires ONCOKB_TOKEN.

#### oncokb_variant_annotation

- **URL**: `https://demo.oncokb.org/api/v1/annotate/mutations/byProteinChange`
- **Description**: OncoKB demo API for variant-level annotations with clinical actionability
- **Data Types**: gene_annotations, cancer_mutations
- **Rate Limit**: Not specified
- **Authentication**: None (demo server). Set ONCOKB_TOKEN for production access.
- **Compliance Notes**: Public MSK OncoKB demo service, works for BRAF/ROS1/TP53 variants. Production (www.oncokb.org) requires ONCOKB_TOKEN.

### Regulatory Data

#### fda_drug_shortages

- **URL**: `https://www.fda.gov/media/169066/download`
- **Description**: FDA Drug Shortages database (cached locally)
- **Data Types**: drug_labels
- **Rate Limit**: Cached with 24-hour TTL
- **Authentication**: None required
- **Compliance Notes**: Public FDA service, drug shortage status information

#### openfda_device_events

- **URL**: `https://api.fda.gov/device/event.json`
- **Description**: FDA MAUDE database for medical device adverse events
- **Data Types**: device_events
- **Rate Limit**: 40 requests/minute (240 with API key)
- **Authentication**: Optional OPENFDA_API_KEY for increased rate limits
- **Compliance Notes**: Public FDA service, device malfunction and adverse event reports

#### openfda_drug_enforcement

- **URL**: `https://api.fda.gov/drug/enforcement.json`
- **Description**: FDA Enforcement database for drug recall information
- **Data Types**: adverse_events
- **Rate Limit**: 40 requests/minute (240 with API key)
- **Authentication**: Optional OPENFDA_API_KEY for increased rate limits
- **Compliance Notes**: Public FDA service, drug recall and enforcement actions

#### openfda_drug_events

- **URL**: `https://api.fda.gov/drug/event.json`
- **Description**: FDA Adverse Event Reporting System (FAERS) for drug safety data
- **Data Types**: adverse_events
- **Rate Limit**: 40 requests/minute (240 with API key)
- **Authentication**: Optional OPENFDA_API_KEY for increased rate limits
- **Compliance Notes**: Public FDA service, voluntary adverse event reports, no PII

#### openfda_drug_labels

- **URL**: `https://api.fda.gov/drug/label.json`
- **Description**: FDA Structured Product Labeling (SPL) for drug prescribing information
- **Data Types**: drug_labels
- **Rate Limit**: 40 requests/minute (240 with API key)
- **Authentication**: Optional OPENFDA_API_KEY for increased rate limits
- **Compliance Notes**: Public FDA service, official drug labeling data

#### openfda_drugsfda

- **URL**: `https://api.fda.gov/drug/drugsfda.json`
- **Description**: FDA Drugs@FDA database for drug approval information
- **Data Types**: drug_labels
- **Rate Limit**: 40 requests/minute (240 with API key)
- **Authentication**: Optional OPENFDA_API_KEY for increased rate limits
- **Compliance Notes**: Public FDA service, official drug approval records

### Functional Enrichment

#### enrichr_addlist

- **URL**: `https://maayanlab.cloud/Enrichr/addList`
- **Description**: Enrichr API for submitting gene lists for enrichment analysis
- **Data Types**: gene_annotations
- **Rate Limit**: Not specified
- **Compliance Notes**: Public Ma'ayan Lab service, gene list metadata only

#### enrichr_enrich

- **URL**: `https://maayanlab.cloud/Enrichr/enrich`
- **Description**: Enrichr API for retrieving functional enrichment results (pathways, ontologies, cell types)
- **Data Types**: pathway_enrichment, ontology_enrichment, gene_annotations
- **Rate Limit**: Not specified
- **Compliance Notes**: Public Ma'ayan Lab service, functional annotation data. Inspired by gget enrichr (Luebbert & Pachter, 2023)

### Czech Healthcare / České zdravotnictví

#### sukl_dlp_api

- **URL**: `https://prehledy.sukl.cz/dlp/v1`
- **Description**: SUKL Drug Registry API for Czech drug information, SPC, PIL, and availability
- **Data Types**: drug_registry, drug_labels
- **Rate Limit**: Not specified
- **Compliance Notes**: Public Czech government API (SUKL), no PII transmitted

#### nrpzs_api

- **URL**: `https://nrpzs.uzis.cz/api/v1`
- **Description**: National Registry of Healthcare Providers API for provider search and details
- **Data Types**: healthcare_providers
- **Rate Limit**: Not specified
- **Compliance Notes**: Public Czech government API (UZIS), provider registry data

#### mkn10_claml

- **URL**: `https://mkn10.uzis.cz`
- **Description**: Czech ICD-10 (MKN-10) classification in ClaML XML format
- **Data Types**: diagnosis_codes
- **Rate Limit**: N/A (local XML file)
- **Compliance Notes**: Public Czech government data (UZIS), diagnostic classification

#### nzip_open_data_api

- **URL**: `https://nzip.cz/api/v3`
- **Description**: NZIP Open Data API for health procedure list (SZV)
- **Data Types**: health_procedures
- **Rate Limit**: Not specified
- **Compliance Notes**: Public Czech government API (NZIP/MZ CR), procedure metadata

#### szv_mzcr

- **URL**: `https://szv.mzcr.cz`
- **Description**: Ministry of Health procedure list for medical procedure codes and point values
- **Data Types**: health_procedures
- **Rate Limit**: Not specified
- **Compliance Notes**: Public Czech government service (MZ CR), procedure definitions

#### vzp_codebooks

- **URL**: `https://www.vzp.cz`
- **Description**: General Health Insurance Company codebooks for reimbursement codes
- **Data Types**: insurance_codebooks
- **Rate Limit**: Not specified
- **Compliance Notes**: Public Czech insurance company data, codebook metadata

## Domain Summary

| Domain                       | Category              | Endpoints |
| ---------------------------- | --------------------- | --------- |
| api.biorxiv.org              | biomedical_literature | 2         |
| api.fda.gov                  | regulatory_data       | 5         |
| api.gdc.cancer.gov           | variant_databases     | 2         |
| clinicaltrials.gov           | clinical_trials       | 1         |
| clinicaltrialsapi.cancer.gov | clinical_trials       | 5         |
| demo.oncokb.org              | cancer_genomics       | 3         |
| maayanlab.cloud              | functional_enrichment | 2         |
| mychem.info                  | variant_databases     | 2         |
| mydisease.info               | variant_databases     | 2         |
| mygene.info                  | variant_databases     | 2         |
| myvariant.info               | variant_databases     | 2         |
| rest.ensembl.org             | variant_databases     | 1         |
| www.cbioportal.org           | cancer_genomics       | 6         |
| www.ebi.ac.uk                | biomedical_literature | 1         |
| www.fda.gov                  | regulatory_data       | 1         |
| www.ncbi.nlm.nih.gov         | biomedical_literature | 3         |
| prehledy.sukl.cz             | czech_healthcare      | 1         |
| nrpzs.uzis.cz               | czech_healthcare      | 1         |
| mkn10.uzis.cz                | czech_healthcare      | 1         |
| nzip.cz                      | czech_healthcare      | 1         |
| szv.mzcr.cz                  | czech_healthcare      | 1         |
| www.vzp.cz                   | czech_healthcare      | 1         |

## Compliance and Privacy

All endpoints accessed by BioMCP:

- Use publicly available APIs
- Do not transmit personally identifiable information (PII)
- Access only aggregate or de-identified data
- Comply with respective terms of service

## Network Control

For air-gapped or restricted environments, BioMCP supports:

- Offline mode via `BIOMCP_OFFLINE=true` environment variable
- Custom proxy configuration via standard HTTP(S)\_PROXY variables
- SSL certificate pinning for enhanced security
