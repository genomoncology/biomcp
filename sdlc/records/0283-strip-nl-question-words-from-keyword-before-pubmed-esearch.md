---
base: bf46bb5968b3661263006251e22e94eb57176686
head: 4c6fa10672f4929458a9e1e88d0ed64905d8068f
---
BioMCP's `build_pubmed_search_term()` passes `filters.keyword` verbatim to PubMed ESearch. When the agent passes a raw NL question (as happens in BioASQ eval and real agent use), PubMed receives queries like:

Imported from March ticket 283. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/283-strip-nl-question-words-from-keyword-before-pubmed-esearch
