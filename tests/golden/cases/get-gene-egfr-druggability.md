# EGFR - druggability
## Druggability (DGIdb / Open Targets)

Categories: Cell Surface, Clinically Actionable, Drug Resistance, Druggable Genome, Kinase, Tyrosine Kinase
### OpenTargets tractability

| Modality | Tractable | Evidence |
|---|---|---|
| small molecule | yes | Approved Drug, Structure with Ligand, High-Quality Ligand, High-Quality Pocket, Druggable Family |
| antibody | yes | Approved Drug, UniProt loc high conf, GO CC high conf, UniProt loc med conf, UniProt SigP or TMHMM, Human Protein Atlas loc |
| PROTAC | yes | Literature, UniProt Ubiquitination, Database Ubiquitination, Half-life Data, Small Molecule Binder |
| other modality | yes | Advanced Clinical |
### OpenTargets safety liabilities

| Event | Datasource | Effect | Biosample |
|---|---|---|---|
| alopecia | Brennan et al. (2024) | Inhibition/Decrease/Downregulation | - |
| ocular keratitis | Brennan et al. (2024) | Inhibition/Decrease/Downregulation | ocular |
| Hepatotoxicity | Brennan et al. (2024) | Activation/Increase/Upregulation | liver |
| Skin rash | Brennan et al. (2024) | Inhibition/Decrease/Downregulation | skin |
| diarrhoea | Brennan et al. (2024) | Inhibition/Decrease/Downregulation | GI |
| Diarrhea | ClinPGx | - | - |
| AST increase | Brennan et al. (2024) | Inhibition/Decrease/Downregulation | liver |
| regulation of catalytic activity | ToxCast | - | cell-free |
| Drug | Interaction Types | Score | Approved | Sources |
|---|---|---|---|---|
| NECITUMUMAB | antibody, inhibitor | 1.091 | yes | 6 |
| CETUXIMAB | antibody, inhibitor | 0.465 | yes | 13 |
| PANITUMUMAB | antibody, inhibitor | 0.182 | yes | 13 |
More:
  biomcp get gene EGFR pathways   - Reactome/KEGG pathway context
  biomcp get gene EGFR ontology   - GO-style functional enrichment
  biomcp get gene EGFR diseases   - disease associations
  biomcp get gene EGFR diagnostics   - diagnostic tests for this gene from GTR
All:
  biomcp get gene EGFR all
See also:
  biomcp get gene EGFR protein   - deepen into protein function and localization
  biomcp search diagnostic --gene EGFR   - diagnostic tests for this gene
  biomcp search pgx -g EGFR   - pharmacogenomics interactions
  biomcp search variant -g EGFR
  biomcp search article -g EGFR
  biomcp search drug --target EGFR
  biomcp gene trials EGFR

[NCBI Gene](https://www.ncbi.nlm.nih.gov/gene/1956) | [UniProt](https://www.uniprot.org/uniprot/P00533) | [Ensembl](https://www.ensembl.org/Homo_sapiens/Gene/Summary?g=ENSG00000146648) | [OMIM](https://www.omim.org/entry/131550)

