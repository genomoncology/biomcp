---
base: 62fd9d9c
head: 9d18fb8f
---

`list --help` now includes author and phenotype, and the human discover page
states exact paging, compact-preview, full-preview, value-size, and structured
output bounds. The corrected adverse-event batch limitation remains intact;
typed JSON pages stay machine-readable executable templates without duplicated
human prose.

A structural test compares rendered `[ENTITY]` help exactly with the production
typed catalog after removing the five named non-entity pages. Rendered help and
discover output remain within the 160-column terminal boundary. Focused tests,
canonical lint, and independent review passed.
