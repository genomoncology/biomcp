---
base: 7e786d4687fdd87c3c70075f2dad67051382fa16
head: cccbfba4b961612cd2d46b001fb3079e78909394
---
Make the public installer abort before extraction when its release archive checksum cannot be downloaded, parsed, computed, or matched

Imported from March ticket 620. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/620-fail-installation-closed-when-release-checksum-proof-is-unavailable
