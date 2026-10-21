---
base: 1354ec76
head: 80b012a2
---

A shared genomic-coordinate value now carries `coordinate`, `genome_build`,
`source`, and optional provenance. Gene detail and search use it with the
MyGene GRCh38 provider default. Variant search rows label their MyVariant
GRCh37 coordinates, including nested source identities. VariantValidator
normalization returns coordinate objects tied to its GRCh38 response path;
routes without a genomic coordinate do not invent one.

JSON assertions cover search and normalization shapes, and the existing real
MyGene, MyVariant, and VariantValidator captures anchor each decoder. Focused
gene, variant, and normalization tests passed.
