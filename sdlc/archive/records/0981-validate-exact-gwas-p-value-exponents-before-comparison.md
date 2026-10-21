---
base: f051668b
head: dc47a1c7
---

GWAS p-values now validate exact mantissa and exponent input before comparison
or display. Invalid or excessive exponents are rejected without unbounded work,
while valid values below floating-point range retain truthful scientific
notation and deterministic ordering.

Focused parsing, boundary, comparison, filtering, ordering, and rendering tests
passed. The complete release gate passed after the batch.
