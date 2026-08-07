---
base: ae5447da22c1fd4cad425c6713784d8e5b5796f3
head: 7040eb76ef914184a24404f778f2d73858dc5dbe
---
Europe PMC exposes `firstIndexDate` per record — the date the article was first indexed — and PubMed exposes an analogous `EDAT` field via E-utilities. BioMCP's article search parses neither, so the returned records carry only the publication `date`. Publication date and index date can be weeks apart (preprints, in-press records, embargoed data), so the current output gives no honest answer to "how recent is BioMCP's literature index for this query?" A user asking "should I trust BioMCP for this breaking-news query about the daraxonrasib topline data" needs that answer fast — only the index date provides it. The same field also helps with the "why isn't my new paper showing up" question: if the newest index date is a week old, the user knows to wait, not debug their query.

Imported from March ticket 196. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/196-surface-europe-pmc-and-pubmed-first-index-date-in-article-search
