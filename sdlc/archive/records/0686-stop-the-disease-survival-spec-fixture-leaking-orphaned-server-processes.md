---
base: a4d4321acfdef78c444f0a1cc4ad58d12d85b107
head: eff7c012192e04eee944401b78d3b35271d9cb67
---

Added a PID-identity-aware fixture supervisor and adopted it for disease-survival so owner death, timeout, and stale PPID-1 recovery reap the server and its owned root.
The real spec runner now exports its coordinator identity, and lifecycle contracts cover owner death, bounded timeout, stale recovery, and nested setup inheritance.
