---
base: f3d8c7fcb0ab3515e77415b2b3173897e7093e9c
head: 4af322eb1e803816904656b2b19cd41c03c0d13d
---
Move spec assertions that duplicate existing unit tests down to the unit layer, stop specs from invoking cargo, and declare the undeclared CTGov fixture dependency

Imported from March ticket 624. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/624-push-duplicated-spec-proof-down-to-unit-tests-and-declare-hidden-fixture-dependencies
