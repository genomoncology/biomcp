---
base: 9eea4a86
head: 535747d9
---

Carried the stale-cache note: GWAS keeps NoStore unconditionally (a source contract pins it); the ClinGen prefetch inherits both the note scope and the --no-cache flag; TLS trust failures are not retried; the stdio test reads stderr once after a bounded wait. Merged in the 535747d9 chain; CI run 36530006778 covers the
merge.
