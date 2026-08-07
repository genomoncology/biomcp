---
flow: build
priority: 4
---
# Surface Europe PMC and PubMed first index date in article search

Europe PMC exposes `firstIndexDate` per record — the date the article was first indexed — and PubMed exposes an analogous `EDAT` field via E-utilities. BioMCP's article search parses neither, so the returned records carry only the publication `date`. Publication date and index date can be weeks apart (preprints, in-press records, embargoed data), so the current output gives no honest answer to "how recent is BioMCP's literature index for this query?" A user asking "should I trust BioMCP for this breaking-news query about the daraxonrasib topline data" needs that answer fast — only the index date provides it. The same field also helps with the "why isn't my new paper showing up" question: if the newest index date is a week old, the user knows to wait, not debug their query.

Completed under March on 2026-04-16, as March ticket 196. Imported as history when this
repo moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/196-surface-europe-pmc-and-pubmed-first-index-date-in-article-search
