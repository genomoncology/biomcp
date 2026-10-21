---
base: 77b19e3d92814c2390f0ef604c1d014cbfd75e4c
head: 659a3b22
---

Generalized the fixture supervisor from its disease-only recovery command to
one authenticated fixture-kind interface, retaining owner PID/start identity,
token, process-group, canonical parent, root-prefix, and pre-signal identity
checks.

The five remaining routine setup helpers and seven direct setup fixtures now
launch through that supervisor and write the same versioned ownership record.
Their cleanup paths use the shared authenticated record instead of trusting
PID values sourced from environment files. The article federated-timeout
fixture now also uses a bounded owned root and the standard-library Python
runtime.

All 55 routine fixture recovery tests pass, including a real exported-owner
SIGKILL matrix for each of the twelve converted fixtures. Disease-survival's
four lifecycle proofs and the affected article, MyChem, and ComplexPortal
fixture contracts also pass. Production `src/` did not change.
