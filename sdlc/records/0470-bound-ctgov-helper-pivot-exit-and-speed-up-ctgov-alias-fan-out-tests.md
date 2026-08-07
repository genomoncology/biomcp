---
base: 0844b45077fd19f92628a245e89e8496f024e236
head: 370d545cb91dc972d4aa880f2ba266aaf8accec1
---
Two CTGov findings from the 2026-06-29 review sweep share one code area (`entities::trial::search::ctgov`) and must be owned by one ticket so two agents do not edit it in parallel:

Imported from March ticket 470. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/470-bound-ctgov-helper-pivot-exit-and-speed-up-ctgov-alias-fan-out-tests
