---
base: 1354ec76
head: 80b012a2
---

Every production `get variant` route now records the assembly that answered
the coordinate. Explicit and inferred genomic coordinates retain their chosen
build; rsID, gene/protein, and transcript fallback queries record MyVariant's
GRCh37 default and its provenance. Coordinates and biomedical annotations are
otherwise unchanged.

Serialized entity tests prove the coordinate, build, and provider-default
provenance travel together. Existing receipted MyVariant fixtures cover direct
GRCh37/GRCh38 lookup, rsID/search identity, and gene/protein routes. The
focused variant suite passed.
