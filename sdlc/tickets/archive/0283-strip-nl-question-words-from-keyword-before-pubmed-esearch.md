---
flow: build
priority: 9
---
# Strip NL question words from keyword before PubMed ESearch

BioMCP's `build_pubmed_search_term()` passes `filters.keyword` verbatim to PubMed ESearch. When the agent passes a raw NL question (as happens in BioASQ eval and real agent use), PubMed receives queries like:

Completed under March on 2026-04-22, as March ticket 283. Imported as history when this
repo moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/283-strip-nl-question-words-from-keyword-before-pubmed-esearch
