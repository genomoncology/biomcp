---
base: 3dd2180262e8d25af40c32929c9d74cfec6941d3
head: 99a52dee4130a3a612f5db36c673e1b15b316fef
---
Three known ways the routine gate stops being deterministic \u2014 cold-cache\ \ fixture-lifecycle timeouts, orphaned fixture children holding the routine lock,\ \ and a make test contract that still calls live PubTator.

Imported from March ticket 644. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/644-harden-the-deterministic-gate-fixture-lifecycle-orphaned-children-and-the-live-pubtator-dependency
