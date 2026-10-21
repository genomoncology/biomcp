---
base: df87022535f94be13cac1ead753fe216a9f69edd
head: dbc376648dd104281670ef354783028aba3b46dd
---

Consolidated the exact repeated routine fixture results: variant-article
identity now runs once instead of six times, saved JATS rendering once instead
of ten times, and the ClinGen CSpec capture once instead of twice. All existing
expectations consume the shared named result.

All 32 runner/isolation tests and the complete named spec-lint audit pass. The
variant identity page takes 46.87 seconds. Loaded-machine `make spec` fell from
592.42 seconds to 352.51 seconds despite a 73-second HEAD-triggered rebuild in
the latter; comparable warm-binary execution fell to about 279.5 seconds, a
2.1x improvement.
