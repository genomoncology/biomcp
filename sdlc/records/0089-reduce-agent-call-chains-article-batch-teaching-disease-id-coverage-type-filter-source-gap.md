---
base: 77f314214f6ffd3e96143ed73c04d9d5071a9366
head: fdacc45d19b5ec3aa0fa3e1f584ef118a6ece511
---
Analysis of the BioASQ research project (005) revealed that agent call chains are 2-3x longer than necessary. The Arnold Chiari syndrome question required 20 sequential BioMCP calls to answer correctly, but could be reduced to ~8 with existing features the agent wasn't taught, and to ~5-6 with two targeted product fixes.

Imported from March ticket 089. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/089-reduce-agent-call-chains-article-batch-teaching-disease-id-coverage-type-filter-source-gap
