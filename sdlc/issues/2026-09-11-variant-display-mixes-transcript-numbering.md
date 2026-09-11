# Variant display mixes transcript numbering, so a report string match fails

Observed 2026-09-11 while agents ran molecular tumor board work against live data in `experiments/05-pi-botassembly-demos/06-depth-experiment/opus-run/`. Two separate agents hit the same case independently.

`MUTYH` c.1187G>A displays with protein numbering from one transcript and cDNA numbering from another. The clinical report in the case names `c.1187G>A`; the protein consequence is `p.Gly396Asp` on the common transcript and `p.Gly382Asp` as legacy numbering on a different transcript. An agent matching the report's identifier against BioMCP output gets no match, even though the variant is in ClinVar: Variation ID 5294, rs36053993, Pathogenic/Likely pathogenic, two-star, multiple submitters, no conflicts.

A not-found result reads as "nothing to see" rather than "you are looking at the wrong transcript." An agent checking whether a lab's variant-of-uncertain-significance call is current will conclude the variant is not found. One agent only resolved it by going to ClinVar directly.

Worth considering: display both transcript numberings, or accept either as a lookup key and say which transcript the answer is on.
