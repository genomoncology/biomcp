---
base: de336054c2925efc4c16ade6b467151999c18b32
head: 9275fb36d278628c699a64003b58d87398aaa732
---
BioASQ evaluation analysis (research 009) found that the agent's dominant failure pattern is searching articles 5-9 times with keyword reformulations when a structured command would have answered the question directly. The article search HATEOAS footer only suggests more search filters (`-k`, `-g`, `-d`, `--type`). It never suggests switching to a structured entity command even when the query clearly contains a gene symbol, drug name, or disease term.

Imported from March ticket 202. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/202-add-entity-aware-hateoas-suggestions-to-article-search
