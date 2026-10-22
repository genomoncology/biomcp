# Clinical rulings for the 0.9.4 review dispositions

Date: 2026-10-19. Clinical reviewers checked three choices made during 0.9.2 work without a recorded clinical ruling.

1. Disease abbreviations. `MM` leading as multiple myeloma, `MF` pointing to myelofibrosis, and `HD` refused stay as they are. NCI and SEER sources confirm the readings, and callers refuse identity the sources do not support. No change.
2. Variant headline date. Showing the newest evaluation date among the ClinVar records that carry the shown classification (ticket 2022) matches NCBI's distinction between evaluation and submission dates. No change.
3. Protein change on another transcript. `TP53 R116Q` resolving to p.Arg248Gln with an other-transcript note, and the ClinVar-free `BRCA1 Q1878R` resolving to p.Gln1857Arg, are correct: each has paired versioned transcript projections onto one allele. Keep both. Acceptance must require that same-allele proof; a canonical residue mismatch or out-of-range position alone is not enough. Ticket 2261 adds that guard. Do not refuse non-MANE or ClinVar-free answers wholesale.

Sources: ClinVar VCV000012356.113 (https://www.ncbi.nlm.nih.gov/clinvar/variation/12356/); HGVS nomenclature, RefSeq background 21.1.5 (https://hgvs-nomenclature.org/stable/background/refseq/); MANE (https://www.ncbi.nlm.nih.gov/refseq/MANE/); and `sdlc/decisions/2026-10-18-transcript-aliases-need-the-same-genomic-change.md`.

## CDKN2A Q50P reference mapping, 2026-10-21

Clinical review confirms with high confidence that GRCh37 `NC_000009.11:g.21974678T>G` and GRCh38 `NC_000009.12:g.21974679T>G` represent one allele, ClinGen CA198385 / ClinVar 187713. Its MANE Select projection is `NM_000077.5:c.149A>C`, protein `NP_000068.1:p.Gln50Pro`. The numerical difference comes from assemblies, not a general minus-strand adjustment. SPDI is zero-based; HGVS genomic numbering is one-based. Preserve reference/transcript pairing; do not relabel the GRCh37 position as GRCh38 or infer allele identity from a protein string alone.

Sources retrieved 2026-10-21: [ClinVar VCV000187713.16](https://www.ncbi.nlm.nih.gov/clinvar/variation/187713/), [ClinGen CA198385](https://reg.genome.network/allele/CA198385), [HGVS general recommendations 21.1.5](https://hgvs-nomenclature.org/stable/recommendations/general/), [HGVS numbering](https://hgvs-nomenclature.org/stable/background/numbering/), and [NCBI SPDI convention](https://ncbiinsights.ncbi.nlm.nih.gov/2017/02/09/new-web-services-for-comparing-and-grouping-sequence-variants/).

Disposition: j4144 returns the correct GRCh37 position from an identity that retains GRCh37 internally. Withdraw 2345 with its actual command/output evidence. Missing serialized assembly/transcript context does not justify a speculative coordinate shift or authorize an interface expansion.

## BRCA1 direct-only expert papers, 2026-10-21

Clinical review confirms with high confidence that PMID10486320 and PMID31788995 directly report the supplied BRCA1 Pro1749Arg allele. Gayther et al.1999 (DOI10.1086/302583), Table1 and Results, pairs codon1749/Pro1749Arg with historical5365C>G. Lai et al.2020 (DOI10.3802/jgo.2020.31.e5), Table3, pairs c.5246C>G/p.Pro1749Arg in a sequenced study specimen. These are study results, not gene co-occurrence or reference-only mentions.

Same-allele evidence: ClinVar VCV000055479.6 pairs U14680.1:n.5365C>G, NM_007294.4:c.5246C>G and NP_009225.1:p.Pro1749Arg. ClinGen CA003387, MANE1.3 Select, pairs this allele with GRCh38 NC_000017.11:g.43057083G>C and GRCh37 NC_000017.10:g.41209100G>C. The paper main texts do not supply versioned transcript/assembly metadata; current registry mappings are not attributed to the papers. Preserve the standing same-allele proof rule and no gene-token fallback.

Sources: [Gayther full text](https://pmc.ncbi.nlm.nih.gov/articles/PMC1288234/), [Lai fulltext XML](https://www.ebi.ac.uk/europepmc/webservices/rest/PMC6918895/fullTextXML), [ClinVar55479](https://www.ncbi.nlm.nih.gov/clinvar/variation/55479/), [ClinGenCA003387](https://reg.genome.network/allele/CA003387). Retrieved2026-10-21. Corrections to the paper tables or registry bridge would change the ruling.
