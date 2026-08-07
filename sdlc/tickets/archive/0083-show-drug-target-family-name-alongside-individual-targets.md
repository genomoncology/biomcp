---
flow: build
priority: 5
---
# Show drug target family name alongside individual targets

`get drug olaparib` shows "Targets: PARP1, PARP2, PARP3" but not the family name "PARP" or "poly(ADP-ribose) polymerase." When BioASQ asks "what is the target of Olaparib?" the gold answer is "PARP" -- the family, not the individual paralogs.

Completed under March on 2026-03-29, as March ticket 083. Imported as history when BioMCP
moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/083-show-drug-target-family-name-alongside-individual-targets
