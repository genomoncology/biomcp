---
base: e5e325137359f80316a5b22f80121713630e91e2
head: 01ca5723e543240f6c0c119f82fbe47f820a8d5f
---
The article full-text/asset resolver ladder collapses source failure into confident absence. `src/entities/article/fulltext.rs` maps HTTP failure, body overflow, and HTML/PDF conversion failure to the same `Miss` state as a healthy absence; if no later rung succeeds, the public card states the consulted sources returned no full text, erasing whether retrieval actually completed (reproduced, high severity, at HEAD `e56630be`; the same contract class ticket 402 intended to harden). Separately, PMC OA archive expansion is unbounded, and the PMC OA→Europe PMC→Figshare asset resolver lacks deterministic coverage that each failure kind survives as `source_unavailable` rather than `not_found`.

Imported from March ticket 582. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/582-preserve-article-full-text-and-asset-resolver-ladder-outcomes-with-bounds-and-tests
