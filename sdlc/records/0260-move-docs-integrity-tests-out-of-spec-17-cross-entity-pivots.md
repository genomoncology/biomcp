---
base: 2abf832b5b960e086a995328fba4bde2bbff145a
head: 2b245c356f83d995ac144408c3f6153afee46d87
---
`spec/17-cross-entity-pivots.md` mixes two unrelated concerns. It has six tests that validate documentation files and not CLI behavior: `Guide page`, `Docs navigation`, `README entry point`, `Docs home entry point`, `First query entry point`, `Quick reference entry point`. These assert that specific strings appear in `README.md`, `docs/index.md`, and related docs pages. The rest of the file tests real CLI pivot output: `Variant pivots`, `Drug to Trials`, `Disease to Drugs`, etc.

Imported from March ticket 260. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/260-move-docs-integrity-tests-out-of-spec-17-cross-entity-pivots
