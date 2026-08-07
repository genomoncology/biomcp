---
flow: build
priority: 10
---
# Reject contradictory protein coordinates in exact variant search and pivots

Exact protein filters can currently return a different residue while presenting the caller's requested alias as if it matched. On current `main`, `search variant -g BRCA1 --hgvsp p.Met1783Ile` returns three `p.M16I` rows labelled `BRCA1 M1783I`; `search variant -g MSH2 --hgvsp p.Leu341Pro` returns `p.L275P` and `p.L407P`. An exact-variant literature pivot built on this join can attach evidence to the wrong allele. A false identity join is more dangerous than a clear unresolved result.

Completed under March on 2026-07-20, as March ticket 598. Imported as history when this
repo moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/598-reject-contradictory-protein-coordinates-in-exact-variant-search-and-pivots
