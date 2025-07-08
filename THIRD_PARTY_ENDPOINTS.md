# Third-Party Endpoints Used by BioMCP

_This file is auto-generated from the endpoint registry._

## Overview

BioMCP connects to 8 external domains across 18 endpoints.

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

## Domain Summary

| Domain               | Category              | Endpoints |
| -------------------- | --------------------- | --------- |
| api.biorxiv.org      | biomedical_literature | 2         |
| api.gdc.cancer.gov   | variant_databases     | 2         |
| clinicaltrials.gov   | clinical_trials       | 1         |
| myvariant.info       | variant_databases     | 2         |
| rest.ensembl.org     | variant_databases     | 1         |
| www.cbioportal.org   | cancer_genomics       | 6         |
| www.ebi.ac.uk        | biomedical_literature | 1         |
| www.ncbi.nlm.nih.gov | biomedical_literature | 3         |

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
