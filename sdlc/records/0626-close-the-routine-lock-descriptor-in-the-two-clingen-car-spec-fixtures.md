---
base: caa3ab4ea273bab8765a1c136f38e3a72bdfec98
head: c5f50dbb3a41b24d7fa433ae1e9bffed58dfce78
---
Add the missing 8>&- redirection to all five leaking fixture spawns so an orphaned fixture server cannot hold the routine spec lock

Imported from March ticket 626. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/626-close-the-routine-lock-descriptor-in-the-two-clingen-car-spec-fixtures
