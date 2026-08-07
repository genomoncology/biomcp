---
base: 28a65c636c6ea9525ddb3a7a016778cad453820d
head: 4c846445f5f08f4946f4c948d113571802df7c47
---
An agent retrieving literature for seven exact variants had to author a 36-query shell matrix and consumed 414,154 model tokens. BioMCP already has compact article search and `article batch`, but `variant articles` accepts one free-form identifier, exposes no `--debug-plan`, and cannot accept structured transcript/genomic identity for several variants in one request. The missing orchestration surface makes callers duplicate alias expansion, lose provenance, and pay repeatedly for shell/JSON plumbing.

Imported from March ticket 602. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/602-add-bounded-batch-variant-literature-retrieval-with-compact-route-plans
