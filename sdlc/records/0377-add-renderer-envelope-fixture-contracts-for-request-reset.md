---
base: 2f1df4c77d201920020fe4a37f3c401c9ff45915
head: ab865cbcaec22e9c2c1877244efd8d1e1d611cae
---
What is IN scope: - Renderer/envelope tests for disease, discover, article, and variant fixture models - `src/render/markdown/*`, `src/render/json/*`, and provenance helpers only where needed for testability or minimal ownership cleanup - `src/entities/discover.rs`, `src/entities/disease/*`, `src/entities/article/*`, and `src/entities/variant/*` only where needed to expose fixture result models or avoid markdown-specific dependencies in new request-contract paths - JSON `_meta.next_commands`, `_meta.source_status`, evidence/provenance, and markdown structural anchors

Imported from March ticket 377. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/377-add-renderer-envelope-fixture-contracts-for-request-reset
