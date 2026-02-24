# Integration Diagram

The diagram below shows the full BioMCP integration architecture: 13 entity types
connected to 29 upstream biomedical data sources, all accessible through a unified
command grammar.

![BioMCP Integration Architecture](../assets/integration-diagram.svg)

## Entity types

| Entity | Primary Sources | Icon |
|--------|----------------|------|
| Gene | MyGene.info, OpenTargets, CIViC, UniProt, STRING, Enrichr, QuickGO, Reactome | DNA |
| Variant | MyVariant.info, MyGene.info, CIViC, cBioPortal, GWAS Catalog, OncoKB, AlphaGenome | Atom |
| Protein | UniProt, InterPro, MyGene.info, STRING | Flask |
| Pathway | Reactome, g:Profiler | Graph |
| Disease | MyDisease.info, OpenTargets, CIViC, HPO, Monarch, Reactome | Virus |
| Drug | MyChem.info, ChEMBL, OpenTargets, CIViC, OpenFDA | Pill |
| Adverse Event | OpenFDA | First Aid |
| PGx | CPIC, PharmGKB | Prescription |
| Article | Europe PMC, PubTator, NCBI ID Converter, PMC OA | Article |
| Trial | ClinicalTrials.gov, NCI CTS | Clipboard |
| Biomarker | NCI CTS | Crosshair |
| Intervention | NCI CTS | Syringe |
| Organization | NCI CTS | Hospital |

## Source categories

- **Genomics** (4): MyGene.info, MyVariant.info, AlphaGenome, GWAS Catalog
- **Protein & Pathway** (7): UniProt, InterPro, STRING, Reactome, QuickGO, g:Profiler, Enrichr
- **Clinical Oncology** (3): CIViC, OncoKB, cBioPortal
- **Disease & Phenotype** (4): MyDisease.info, HPO, Monarch, OpenTargets
- **Drug & Safety** (5): MyChem.info, ChEMBL, OpenFDA, CPIC, PharmGKB
- **Literature** (4): Europe PMC, PubTator, NCBI ID Converter, PMC OA
- **Clinical Trials** (2): ClinicalTrials.gov, NCI CTS

Icons in the diagram are from [Phosphor Icons](https://phosphoricons.com).

See [Data Sources](../reference/data-sources.md) for authentication requirements, base
URLs, and operational details for each source.
