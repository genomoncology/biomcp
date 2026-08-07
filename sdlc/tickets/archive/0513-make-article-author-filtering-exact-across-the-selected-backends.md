---
flow: build
priority: 5
---
# Make article author filtering exact across the selected backends

`search article -a/--author` is documented by clap as “Filter by author name,” but the default federated route sends the name as a real author-field query to Europe PMC/PubMed and as free text to PubTator/Semantic Scholar, then unions all rows. `Williams LS` therefore returns Williams syndrome and unrelated lexical matches ahead of real byline matches. The flag is also absent from `biomcp list article`. Backend query syntax placed in `-k` is provider-neutralized inconsistently instead of being honored or rejected.

Completed under March on 2026-07-14, as March ticket 513. Imported as history when BioMCP
moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/513-make-article-author-filtering-exact-across-the-selected-backends
