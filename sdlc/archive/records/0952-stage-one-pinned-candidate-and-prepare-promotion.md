---
base: d40e2cf28ae4086005fc0fb96efce0a35024e73d
head: 5ec8a2ba11b68d287ed7d26e6227263f6df9b332
---

Added a checksummed private candidate manifest that binds one committed version,
full main SHA, stage run, tool pins, gates, and exact artifact bytes. Candidate
registration is strict, complete-set sealing is fail-closed, and the initial
workflow has no public write path.

Deterministic native and wheel packaging, archive inspection, replay/conflict
tests, manifest schema validation, and the private stage workflow establish the
immutable input later release tickets consume.
