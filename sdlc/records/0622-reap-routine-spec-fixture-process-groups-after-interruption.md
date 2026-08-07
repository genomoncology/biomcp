---
base: ed07905ec70852f468954456c2f1cd56e335a334
head: 5147c0c657dd20839533a38c523320ca89f24cb2
---
Make interrupted routine specs terminate only their own fixture process groups and prevent stale children from holding the shared lock

Imported from March ticket 622. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/622-reap-routine-spec-fixture-process-groups-after-interruption
