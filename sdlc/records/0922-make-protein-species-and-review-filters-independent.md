---
base: 9259bc2c4ab8379602f666322ebb82dbe4a9692b
head: dcbc1d94d6f5e65559e6e2de96946d7620cf3df8
---

Protein search defaults to reviewed human entries. `--all-species` changes
only the species restriction, `--include-unreviewed` changes only review
status, and incompatible review flags fail before a provider request. Returned
rows expose their reviewed status.

Independent filter combinations, request plans, help text, JSON, Markdown,
and deterministic UniProt scenarios all passed.
