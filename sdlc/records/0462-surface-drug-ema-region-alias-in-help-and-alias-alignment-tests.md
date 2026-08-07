---
base: e37a3a4af0bf3313d24166cd744340a58c2c1cca
head: dae6f6930d49c43b6c7d4181fe33834c30279004
---
`search drug --region ema` works and `biomcp list drug` documents the alias, but `search drug --help` advertises only `us, eu, who, all`. Accepted public aliases should be visible in help/list/docs/specs together; hidden aliases confuse users and break the “one CLI contract” rule.

Imported from March ticket 462. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/462-surface-drug-ema-region-alias-in-help-and-alias-alignment-tests
