---
base: ea341fe5674dc146d1f8bfeb40bd331e3a6d46e0
head: 99b605082bc8e5836cad062bc634c8ebda85f94d
---
Research 009 (BioASQ skill optimization) ran 60 questions through BioMCP with an optimized skill. The biggest single source of wasted agent calls is drug resolution failure and missing search fallbacks. Three specific gaps caused ~85 wasted calls across 60 questions:

Imported from March ticket 092. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/092-improve-drug-resolution-and-search-fallbacks-for-agent-efficiency
