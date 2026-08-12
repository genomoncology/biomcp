---
base: 1354ec76
head: 80b012a2
---

Human variant detail now renders `Genomic coordinate (BUILD): value`, adding
`provider default` when that is how the assembly was chosen. Variant search
has a compact Build column. Gene detail labels its coordinate directly and
gene search separates coordinate and build columns. Normalization renders the
assembly from its typed coordinate rather than hard-coding GRCh38.

Requested gene/protein, gene/coding, transcript, rsID, and genomic identities
now render as readable phrases instead of serialized JSON. Renderer tests
cover provider defaults, search tables, normalization, and legacy fallbacks.
The combined 0950/0899/0900 implementation added 155 net Rust/template lines,
below the tickets' combined 270-line ceiling.
