---
base: b05211250e5a0b3c66e2e549acf340d05c94e7c6
head: 52423046807817fd5fee5a8e3a406686923c4b24
---
PubMed federation is wired and working, but PubMed-unique papers get buried in federated results. The directness ranker scores papers by how many query terms appear in the title/abstract, but PubMed's value is finding papers that use *different* terminology than the query — MeSH headings, author keywords, and NLM indexer terms. A paper found only by PubMed with none of the query words in its title scores lowest in the current ranker and falls off the visible top-N.

Imported from March ticket 146. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/146-federated-article-ranking-fails-to-surface-pubmed-unique-hits
