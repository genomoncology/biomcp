---
base: b8ac041281b447bab62047d63ad722b9bc8403e0
head: 316fb7788176efb373950d583cd439ba86942509
---
Routine `make spec` still runs `spec/entity/pathway.md` live against KEGG / Reactome / WikiPathways with no fixture, so it flakes on upstream ranking drift and availability (the WikiPathways 404 lineage). It is the one live entity spec the 376–379 fixture/quarantine effort never moved out of the routine pool — disease, discover, and protein are already deselected/serialized, but pathway runs in the parallel pool. Because the build flow's design step establishes its red baseline by running full `make spec`, these 2 pathway failures block every spec-gated ticket (currently ticket 385) from proving new assertions red for the right reason. Quarantining pathway out of routine `make spec` makes the routine behavioral gate deterministic, consistent with the test-strategy-reset already applied to the other live entity specs.

Imported from March ticket 390. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/390-quarantine-live-pathway-spec-out-of-routine-make-spec
