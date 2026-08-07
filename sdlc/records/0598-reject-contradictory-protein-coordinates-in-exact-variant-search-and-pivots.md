---
base: 4495360eb6f68cfb1ae6d3cca31d1be0bacfffef
head: 2a8a05576929524e2b175fe201b985782295ed95
---
Exact protein filters can currently return a different residue while presenting the caller's requested alias as if it matched. On current `main`, `search variant -g BRCA1 --hgvsp p.Met1783Ile` returns three `p.M16I` rows labelled `BRCA1 M1783I`; `search variant -g MSH2 --hgvsp p.Leu341Pro` returns `p.L275P` and `p.L407P`. An exact-variant literature pivot built on this join can attach evidence to the wrong allele. A false identity join is more dangerous than a clear unresolved result.

Imported from March ticket 598. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/598-reject-contradictory-protein-coordinates-in-exact-variant-search-and-pivots
