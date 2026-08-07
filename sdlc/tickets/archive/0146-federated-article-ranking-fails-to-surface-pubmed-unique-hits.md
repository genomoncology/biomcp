---
flow: architect
priority: 7
---
# Federated article ranking fails to surface PubMed-unique hits

PubMed federation is wired and working, but PubMed-unique papers get buried in federated results. The directness ranker scores papers by how many query terms appear in the title/abstract, but PubMed's value is finding papers that use *different* terminology than the query — MeSH headings, author keywords, and NLM indexer terms. A paper found only by PubMed with none of the query words in its title scores lowest in the current ranker and falls off the visible top-N.

Completed under March on 2026-04-04, as March ticket 146. Imported as history when BioMCP
moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/146-federated-article-ranking-fails-to-surface-pubmed-unique-hits
