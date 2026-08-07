---
base: fe85b1b2d5a412a6d0b493130a0083767a58d8aa
head: 2cd35df47b1626fa90c7bb5c35d503bdd35688e2
---
The PubMed rescue heuristic (ticket 150) promotes PubMed-unique or PubMed-led rows above rows from other backends when the PubMed result is weak on lexical directness. This works well for the core use case — a paper PubMed found via MeSH synonyms that shares at least one query term. However, the rescue has no lexical-overlap floor. A PubMed-unique row with zero anchor hits (no query terms in title or abstract) gets promoted above tier-1 rows that match 3/4 anchors, because the rescue flag sorts above directness tier in the comparator.

Imported from March ticket 151. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/151-guard-pubmed-rescue-against-zero-overlap-false-positives
