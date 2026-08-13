---
base: c98ca941
head: 61474ad5
---

GWAS Catalog p-values now preserve exact scientific notation, mantissa, and
exponent in a structured value. Values below `f64` range keep `numeric: null`,
while exact comparison drives filtering and ordering and Markdown renders the
truthful scientific value.

Mixed-result batch reports remain stdout outcomes with their nonzero process
status instead of becoming an `Error:` wrapper; MCP retains their text report.
A bounded MyGene.info identity lookup promotes a unique canonical gene alias,
including EGFR for ERBB1, without requiring an HGNC result in OLS4's window.
Full-text files use private creation and atomic replacement and normal managed
state access repairs older permissive files.

Focused Rust tests covered exact underflow serialization/comparison, mixed
batch settlement and MCP projection, missing-HGNC alias ranking, and new and
existing download permissions.
