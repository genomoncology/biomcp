---
base: af748fb6bb1a90839c6f66899f6945534c4859b0
head: 3bc2f48609f1fc4f80145bcf11875c008eee2f53
---
Project 38 BioASQ benchmark rounds repeatedly hit Semantic Scholar rate-limit timeouts during BioMCP article search. This materially degrades answer recall on list questions and makes it hard to distinguish BioMCP product retrieval gaps from benchmark harness/environment gaps.

Imported from March ticket 364. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/364-architect-semantic-scholar-timeout-and-credential-propagation-contract
