# Transcript aliases need the same genomic change

Date: 2026-10-18. Source: clinical review ruling requested for 2185.

## Ruling

Variant article search accepts an alias from another transcript as the searched variant only when the paper itself pins the alias to the same genomic change through at least one of:

- a stated transcript accession and version whose projection gives the searched change;
- genomic coordinates with a named genome build;
- an rsID or ClinGen allele ID that resolves to the same allele.

Otherwise a paper matched only by the alias stays unverified, whatever the alias string.

## Behavior

- Keep generating aliases, each tagged with its source transcript.
- Show verified papers first. Show alias-only matches in a separate group labeled "unverified alias match", naming the alias and its transcript.
- When an alias string is also a valid, different variant on the MANE Select transcript, label the match as a possible different variant, or drop it.
- Never count alias-only matches as verified, and never fold them into evidence summaries.

Example: for BRCA1 F1761I, the aliases c.211T>A and F71I are invalid against NM_007294.4, where c.211 is A and codon 71 is arginine.

## Sources

- HGVS general recommendations: a variant is described against a stated reference sequence that contains the changed residue; MANE is preferred.
- Richards et al. 2015, ACMG/AMP standards: give the reference transcript and weigh all clinically relevant transcripts.
- Li et al. 2017, AMP/ASCO/CAP somatic variant guideline: use transcript accession and version; prefer genomic coordinates with a build.
- Morales et al. 2022, MANE: laboratories choosing their own transcripts causes discordant variant identification.
