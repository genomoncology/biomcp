---
base: 83af29b36f2ca95ac15a7c98cd93078f5d210c40
head: cfba7a5f6680f38203657874b2c7f20c2c5dd9b0
---
The non-finite-float validation gap (known in variant `--gerp-min`/`--min-cadd`, ticket 588) recurs in other commands, per the 2026-07-18 fuzz sweep (`experiments/161-biomcp-adversarial-input-fuzz/FINDINGS.md`):

Imported from March ticket 595. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/595-reject-non-finite-and-out-of-range-numeric-filters-across-gwas-trial-and-discover
