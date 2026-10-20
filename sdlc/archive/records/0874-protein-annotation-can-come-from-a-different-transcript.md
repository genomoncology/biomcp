---
base: 9259bc2c4ab8379602f666322ebb82dbe4a9692b
head: 1e963c32ff1674731d10e25e54aa8030eb1a7052
---

Variant detail and search now select transcript, coding HGVS, and protein HGVS
from one transcript-associated annotation. The request includes paired
snpEff and ClinVar evidence, canonical RefSeq evidence is preferred, and
independently ordered dbNSFP arrays are never zipped or selected as a pair.

Adversarial and no-pair tests prove mismatched protein identities are omitted.
Real BRAF and PTEN receipts pass through the production parser, BRAF V600E no
longer becomes V640E, and the JSON, Markdown, blog, and routine specification
contracts all passed.
