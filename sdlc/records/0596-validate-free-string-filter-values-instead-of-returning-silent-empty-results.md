---
base: 15fa53fc11f11f018f82b577ba17ae1cf81d76a0
head: 516c030d2e20ff051699d670641f7db700f5bbec
---
The 2026-07-18 fuzz sweep (`experiments/161-biomcp-adversarial-input-fuzz/FINDINGS.md`) found a cluster of free-string filters that silently accept unvalidated garbage and return a successful **empty** result (confident emptiness), while sibling filters on the same commands validate and error:

Imported from March ticket 596. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/596-validate-free-string-filter-values-instead-of-returning-silent-empty-results
