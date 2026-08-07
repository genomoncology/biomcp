---
flow: build
priority: 7
---
# Reject non-finite and out-of-range numeric filters across gwas, trial, and discover

The non-finite-float validation gap (known in variant `--gerp-min`/`--min-cadd`, ticket 588) recurs in other commands, per the 2026-07-18 fuzz sweep (`experiments/161-biomcp-adversarial-input-fuzz/FINDINGS.md`):

Completed under March on 2026-07-19, as March ticket 595. Imported as history when this
repo moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/595-reject-non-finite-and-out-of-range-numeric-filters-across-gwas-trial-and-discover
