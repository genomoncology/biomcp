---
base: 38ce2254e7047971d58b11f1e43176af503f2c8f
head: 1a492c679479d6ee72458b3926a1a0589910a9f2
---
`biomcp search trial --intervention venetoclax` fails completely with a ClinicalTrials.gov HTTP 400. Alias expansion takes MyChem DrugBank synonyms presented as `brand_names`, including long IUPAC names with brackets, and sends each alias directly to CTGov's `query.intr` ESSIE parser. `--no-alias-expand` succeeds, proving the failure is introduced by BioMCP. This is a common clinical drug path and must not depend on the punctuation in upstream synonym data.

Imported from March ticket 510. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/510-keep-ctgov-intervention-aliases-literal-and-trial-safe
