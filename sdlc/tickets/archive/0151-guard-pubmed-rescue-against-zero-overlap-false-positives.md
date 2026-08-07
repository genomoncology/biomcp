---
flow: build
priority: 9
---
# Guard PubMed rescue against zero-overlap false positives

The PubMed rescue heuristic (ticket 150) promotes PubMed-unique or PubMed-led rows above rows from other backends when the PubMed result is weak on lexical directness. This works well for the core use case — a paper PubMed found via MeSH synonyms that shares at least one query term. However, the rescue has no lexical-overlap floor. A PubMed-unique row with zero anchor hits (no query terms in title or abstract) gets promoted above tier-1 rows that match 3/4 anchors, because the rescue flag sorts above directness tier in the comparator.

Completed under March on 2026-04-05, as March ticket 151. Imported as history when BioMCP
moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/151-guard-pubmed-rescue-against-zero-overlap-false-positives
