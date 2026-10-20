---
base: f64f103fe33691d317fdbf5648e6897782cced2f
head: 5751a1085326f89fb4a7b481851e330d0d880fa8
---

# Survival fixture stops reading /proc after exit

Lifecycle coverage now uses health endpoint refusal and heartbeat progress or
stoppage rather than post-exit procfs entries. This preserves owner-death,
timeout cleanup, authenticated orphan recovery, and decoy noninterference
proofs without depending on zombie reaping timing.
