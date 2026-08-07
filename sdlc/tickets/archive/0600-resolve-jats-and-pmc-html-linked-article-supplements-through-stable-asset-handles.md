---
flow: build
priority: 8
---
# Resolve JATS- and PMC-HTML-linked article supplements through stable asset handles

BioMCP's JATS converter can display supplement filenames that the asset resolver cannot retrieve. On current `main`, article text for PMID 20516115 names two supplements (`Supplementary_Methods__Figures__Tables.pdf` and `Supplementary_Tables.xls`), while `get article 20516115 assets --json` returns no handles. Prior work made the empty/failure outcome more honest and added PMC OA, Europe PMC ZIP, and Figshare sibling retrieval, but the resolver still ignores supplement links carried directly by JATS or PMC HTML.

Completed under March on 2026-07-20, as March ticket 600. Imported as history when this
repo moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/600-resolve-jats-and-pmc-html-linked-article-supplements-through-stable-asset-handles
