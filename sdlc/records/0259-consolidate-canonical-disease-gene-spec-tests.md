---
base: 1c0d484bf16c05e733efb54943779113022fb48b
head: 97aedb52398d9ac1686025e7ca3b8c7ca16a0155
---
`spec/07-disease.md` contains four near-duplicate tests that each call `biomcp get disease <name>` with a different canonical disease and assert a specific gene list: `Canonical CLL Disease Genes`, `Canonical T-PLL Disease Genes`, `Canonical Parkinson Disease Genes`, `Canonical CMT1A Disease Genes`. Each hits OpenTargets with a distinct MONDO ID, so each is a separate cold-cache network request. They duplicate the same CLI contract — "canonical disease card surfaces a disease-genes table" — four times at four different wall-time costs.

Imported from March ticket 259. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/259-consolidate-canonical-disease-gene-spec-tests
