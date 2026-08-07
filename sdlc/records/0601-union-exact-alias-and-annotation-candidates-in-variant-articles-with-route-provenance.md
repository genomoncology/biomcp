---
base: 3cde77b2f1069057af5a15c0e03409da45245178
head: 74ed7d9a283cd39be1bbec6ec7ae35664e3ea980
---
`variant articles` currently chooses one route rather than preserving the union of evidence routes. On current `main`, `variant articles 'MSH2 p.L341P'` returns zero via its best-effort fallback while ordinary exact-keyword article search returns PMID 26951660. `variant articles 'BRCA1 p.M1783I'` returns one annotation paper while exact-keyword search returns PMID 20516115. An annotation hit is useful but incomplete; it must not suppress a lexical route that succeeds in the same binary.

Imported from March ticket 601. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/601-union-exact-alias-and-annotation-candidates-in-variant-articles-with-route-provenance
