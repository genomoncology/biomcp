---
base: a6e4fb7861c217598439d19a5b582aa8a85b5843
head: 4f1a08f38bc73d417f703fd2b87597310627bb3d
---
`get drug olaparib` shows "Targets: PARP1, PARP2, PARP3" but not the family name "PARP" or "poly(ADP-ribose) polymerase." When BioASQ asks "what is the target of Olaparib?" the gold answer is "PARP" -- the family, not the individual paralogs.

Imported from March ticket 083. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/083-show-drug-target-family-name-alongside-individual-targets
