---
title: "Biomedical Data Sources for AI Agents | BioMCP"
description: "Explore BioMCP source guides for PubMed, ClinicalTrials.gov, ClinVar, OpenFDA, CDC WONDER VAERS, UniProt, gnomAD, Reactome, Semantic Scholar, ChEMBL, OpenTargets, SEER Explorer, CIViC, OncoKB, cBioPortal, DDInter, EMA, WHO Prequalification, WHO Prequalified IVD, CDC CVX/MVX, KEGG, PharmGKB / CPIC, Human Protein Atlas, and Monarch Initiative."
---

# Biomedical Data Sources for AI Agents

BioMCP's [User Guide](../user-guide/cli-reference.md) is organized around entities such as genes, variants, articles, trials, and drugs. This Sources section flips the lens: it shows what each upstream database is good at, what BioMCP exposes from it, and where the boundary sits when a workflow is mixed-source.

Use these pages when you already know the provider you trust, the keyword you are targeting, or the provenance you need to explain to a reviewer, teammate, or downstream agent.

## Source guides

| Source | Best when you want | Guide |
|---|---|---|
| PubMed | Article search, PubTator annotations, and PMC full-text handoff | [PubMed](pubmed.md) |
| ClinicalTrials.gov | Recruiting-study search, eligibility text, and site details | [ClinicalTrials.gov](clinicaltrials-gov.md) |
| ClinVar | Clinical significance and review-status context for variants | [ClinVar](clinvar.md) |
| OpenFDA | FAERS, recalls, device events, labels, and U.S. approval context | [OpenFDA](openfda.md) |
| CDC WONDER VAERS | Vaccine adverse-event summaries, seriousness breakdowns, and age-distribution context | [CDC WONDER VAERS](vaers.md) |
| UniProt | Canonical protein cards and structure-linked context | [UniProt](uniprot.md) |
| gnomAD | Population frequency and gene constraint context | [gnomAD](gnomad.md) |
| Reactome | Pathway records, pathway genes, and contained events | [Reactome](reactome.md) |
| Semantic Scholar | TLDRs, citation graphs, references, and recommendations | [Semantic Scholar](semantic-scholar.md) |
| ChEMBL | Drug-target activity, mechanism context, and indication enrichment | [ChEMBL](chembl.md) |
| OpenTargets | Target-disease scores, druggability, and disease-gene evidence | [OpenTargets](opentargets.md) |
| SEER Explorer | Cancer survival statistics and disease `survival` section output | [SEER Explorer](seer.md) |
| CIViC | Clinical variant evidence, therapy context, and disease-associated variants | [CIViC](civic.md) |
| OncoKB | Oncology actionability tiers and treatment implications for actionable variants | [OncoKB](oncokb.md) |
| cBioPortal | Cancer cohort frequencies and local study analytics workflows | [cBioPortal](cbioportal.md) |
| DDInter | Structured drug-drug interactions, severity levels, and class-oriented partner review | [DDInter](ddinter.md) |
| EMA | EU regulatory, safety, and shortage context for medicines | [EMA](ema.md) |
| WHO Prequalification | WHO-backed medicine and vaccine prequalification search plus global access checks | [WHO Prequalification](who-prequalification.md) |
| WHO Prequalified IVD | Infectious-disease diagnostic products, assay formats, and WHO product-card provenance | [WHO Prequalified IVD](who-ivd.md) |
| CDC CVX/MVX | Vaccine brand-to-antigen bridge for EMA/default lookups and explicit WHO vaccine search | [CDC CVX/MVX](cdc-cvx.md) |
| KEGG | KEGG pathway IDs, summary cards, and pathway genes | [KEGG](kegg.md) |
| PharmGKB / CPIC | Pharmacogenomic recommendations, frequencies, and clinical annotations | [PharmGKB / CPIC](pharmgkb.md) |
| Human Protein Atlas | Tissue expression, localization, and cancer-expression context | [Human Protein Atlas](human-protein-atlas.md) |
| Monarch Initiative | Phenotype-to-disease matching, disease genes, and model evidence | [Monarch Initiative](monarch-initiative.md) |

## Reference and setup

- [Data Sources](../reference/data-sources.md) explains runtime behavior, endpoints, auth mode, and operational caveats.
- [Source Licensing and Terms](../reference/source-licensing.md) explains direct vs indirect provenance, redistribution limits, and provider terms.
- [API Keys](../getting-started/api-keys.md) shows the optional or required environment variables that upgrade selected source paths.

## Related docs

- [CLI Reference](../user-guide/cli-reference.md)
- [Article](../user-guide/article.md)
- [Trial](../user-guide/trial.md)
- [Variant](../user-guide/variant.md)
- [Drug](../user-guide/drug.md)
