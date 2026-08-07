---
base: 9f12fcdeccce0f93893db16a2edf981ae0583b44
head: 66e62762c55a33dd8eff9c53b7de81646deb4527
---
Downgrade the cleanup-succeeded-still-under-floor cache WARN to debug; trivial log-level fix.

Imported from March ticket 437. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/437-quiet-noisy-cache-disk-space-warn-when-cleanup-succeeded
