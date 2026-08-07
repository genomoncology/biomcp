---
flow: build
priority: 9
---
# Union exact-alias and annotation candidates in variant articles with route provenance

`variant articles` currently chooses one route rather than preserving the union of evidence routes. On current `main`, `variant articles 'MSH2 p.L341P'` returns zero via its best-effort fallback while ordinary exact-keyword article search returns PMID 26951660. `variant articles 'BRCA1 p.M1783I'` returns one annotation paper while exact-keyword search returns PMID 20516115. An annotation hit is useful but incomplete; it must not suppress a lexical route that succeeds in the same binary.

Completed under March on 2026-07-20, as March ticket 601. Imported as history when BioMCP
moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/601-union-exact-alias-and-annotation-candidates-in-variant-articles-with-route-provenance
