# Source Labels

This spec verifies that `get` entity detail responses carry explicit source
labels in both markdown and JSON output. Assertions are structural — they check
for stable provenance strings, not volatile upstream data values.

| Section | Command focus | Why it matters |
|---|---|---|
| Markdown labels | `get <entity>` | Confirms visible source attribution at section boundaries |
| JSON section_sources | `get <entity> --json` | Confirms `_meta.section_sources` with stable key/label/sources fields |

## Markdown Source Labels

Each entity type must name its upstream source at visible section boundaries.

```bash
gene_out="$(biomcp get gene CFTR all)"
echo "$gene_out" | mustmatch like "Source: NCBI Gene / MyGene.info"
echo "$gene_out" | mustmatch like "## Summary (NCBI Gene)"

drug_out="$(biomcp get drug ivacaftor targets)"
echo "$drug_out" | mustmatch like "## Targets (ChEMBL / Open Targets)"

disease_out="$(biomcp get disease "cystic fibrosis" all)"
echo "$disease_out" | mustmatch like "Genes (Open Targets):"
echo "$disease_out" | mustmatch like "## Pathways (Reactome)"

trial_out="$(biomcp get trial NCT06668103)"
echo "$trial_out" | mustmatch like "Source: ClinicalTrials.gov"

protein_out="$(biomcp get protein P15056 complexes)"
echo "$protein_out" | mustmatch like "## Complexes (ComplexPortal)"

pgx_out="$(biomcp get pgx CYP2D6 recommendations)"
echo "$pgx_out" | mustmatch like "## Recommendations (CPIC)"

ae_out="$(biomcp get adverse-event 10329882)"
echo "$ae_out" | mustmatch like "## Reactions (OpenFDA)"
```

## JSON section_sources — Gene, Drug, Disease

Core entity types must include a non-empty `_meta.section_sources` array.

```bash
gene_json="$(biomcp get gene CFTR all --json)"
echo "$gene_json" | mustmatch like '"section_sources": ['
echo "$gene_json" | mustmatch like '"key": "summary"'
echo "$gene_json" | mustmatch like '"key": "identity"'
echo "$gene_json" | mustmatch like "NCBI Gene"

drug_json="$(biomcp get drug ivacaftor all --json)"
echo "$drug_json" | mustmatch like '"section_sources": ['
echo "$drug_json" | mustmatch like '"key": "safety"'
echo "$drug_json" | mustmatch like '"key": "targets"'
echo "$drug_json" | mustmatch like "OpenFDA FAERS"
echo "$drug_json" | mustmatch like "ChEMBL"

disease_json="$(biomcp get disease "cystic fibrosis" all --json)"
echo "$disease_json" | mustmatch like '"section_sources": ['
echo "$disease_json" | mustmatch like '"key": "definition"'
echo "$disease_json" | mustmatch like '"key": "top_genes"'
echo "$disease_json" | mustmatch like '"key": "recruiting_trials"'
echo "$disease_json" | mustmatch like "MyDisease.info"
echo "$disease_json" | mustmatch like "ClinicalTrials.gov"
```

## JSON section_sources — Variant, Trial, Article

```bash
variant_json="$(biomcp get variant rs334 --json)"
echo "$variant_json" | mustmatch like '"section_sources": ['
echo "$variant_json" | mustmatch like '"key": "identity"'
echo "$variant_json" | mustmatch like "MyVariant.info"

trial_json="$(biomcp get trial NCT06668103 --json)"
echo "$trial_json" | mustmatch like '"section_sources": ['
echo "$trial_json" | mustmatch like '"key": "overview"'
echo "$trial_json" | mustmatch like "ClinicalTrials.gov"

article_json="$(biomcp get article 22663011 --json)"
echo "$article_json" | mustmatch like '"section_sources": ['
echo "$article_json" | mustmatch like '"key": "bibliography"'
echo "$article_json" | mustmatch like "PubMed"
```

## JSON section_sources — Pathway, Protein, PGX, Adverse Event

```bash
pathway_json="$(biomcp get pathway R-HSA-5358351 all --json)"
echo "$pathway_json" | mustmatch like '"section_sources": ['
echo "$pathway_json" | mustmatch like '"key": "identity"'
echo "$pathway_json" | mustmatch like "Reactome"

wp_json="$(biomcp get pathway WP254 --json)"
echo "$wp_json" | mustmatch like '"section_sources": ['
echo "$wp_json" | mustmatch like '"key": "identity"'
echo "$wp_json" | mustmatch like "WikiPathways"

protein_json="$(biomcp get protein P15056 --json)"
echo "$protein_json" | mustmatch like '"section_sources": ['
echo "$protein_json" | mustmatch like '"key": "identity"'
echo "$protein_json" | mustmatch like "UniProt"

pgx_json="$(biomcp get pgx CYP2D6 --json)"
echo "$pgx_json" | mustmatch like '"section_sources": ['
echo "$pgx_json" | mustmatch like "CPIC"

ae_json="$(biomcp get adverse-event 10329882 --json)"
echo "$ae_json" | mustmatch like '"section_sources": ['
echo "$ae_json" | mustmatch like '"key": "reactions"'
echo "$ae_json" | mustmatch like "OpenFDA"
```

## Backward Compatibility

Adding `section_sources` must not break the existing `_meta` contract.

```bash
gene_json="$(biomcp get gene CFTR --json)"
echo "$gene_json" | mustmatch like '"evidence_urls": ['
echo "$gene_json" | mustmatch like '"next_commands": ['
echo "$gene_json" | mustmatch like '"section_sources": ['
```
