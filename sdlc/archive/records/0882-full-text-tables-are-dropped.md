---
base: 6d2ecbee
head: 91634748
---

Complex JATS tables now retain every source row and mark merged cells with
explicit rowspan and colspan annotations. A bounded warning says the visual
layout may be lossy; ordinary tables keep their compact output unchanged.

The real receipted NCBI EFetch response for PMID 30311380 / PMC6329583 passes
through the production XML normalizer and JATS renderer. All six tables and
their identifying content survive in unit and process-level fixture checks.
