---
base: 16d630f8
head: f3bc4220
---

The successful variant-article identity anchor now uses byte-faithful,
receipted ClinGen responses for TP53 NM_000546.6:c.215C>G, CA000072, and the
PMC8372092 table annotation. The production CAR and LDH clients, decoders,
identity merge, route trace, terminal state, work accounting, JSON, and
Markdown paths all consume those captures.

Synthetic rows remain only for deterministic collisions, ordering, bounded
work, and failure cases and are explicitly described as non-evidence. Unknown
provider routes fail, and the receipt audit accepted all 221 source files.
