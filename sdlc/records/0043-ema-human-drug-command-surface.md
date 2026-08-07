---
base: fff62770688daa5386fb10188262fdfaeaade15a
head: 49755194314284df2d4b9d6ba6d31b51a75897fb
---
BioMCP already has useful U.S. drug coverage through sources like OpenFDA, Drugs@FDA, MyChem, and FAERS, but it does not expose a comparable EU regulatory layer. EMA publishes a compact human-medicine batch that is small enough to keep locally and rich enough to answer user-facing questions about EU authorization status, safety communications, and shortages. This ticket adds that EU layer without replacing the current U.S. behavior.

Imported from March ticket 043. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/043-ema-human-drug-command-surface
