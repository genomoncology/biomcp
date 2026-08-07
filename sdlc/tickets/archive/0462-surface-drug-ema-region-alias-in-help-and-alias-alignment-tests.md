---
flow: quickfix
priority: 6
---
# Surface drug ema region alias in help and alias-alignment tests

`search drug --region ema` works and `biomcp list drug` documents the alias, but `search drug --help` advertises only `us, eu, who, all`. Accepted public aliases should be visible in help/list/docs/specs together; hidden aliases confuse users and break the “one CLI contract” rule.

Completed under March on 2026-06-30, as March ticket 462. Imported as history when this
repo moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/462-surface-drug-ema-region-alias-in-help-and-alias-alignment-tests
