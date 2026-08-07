---
base: 29707982ad5232805d8fecf371cfe066d44b3969
head: 785106318ff12fe06dd1f212050df40830e2c273
---
The 2026-07-18 fuzz sweep (`experiments/161-biomcp-adversarial-input-fuzz/FINDINGS.md`) found two `study` analytics commands emit confident-looking statistics on empty or structurally-degenerate groups, where `study survival` already does the right thing (returns `null`):

Imported from March ticket 597. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/597-return-null-instead-of-fabricated-statistics-for-empty-or-degenerate-study-groups
