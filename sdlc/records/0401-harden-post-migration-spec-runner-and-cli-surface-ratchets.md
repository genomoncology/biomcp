---
base: 354401409999d577e1b1fc968532a66290240ed1
head: 607f2bda34dbfdba818a83c88f4f8ac3d922857f
---
The migration made the routine spec lane much healthier, but review found many assertion-strength gaps where contracts can stay green while the user-visible behavior or runner participation regresses. These should become automated spec/lint/contract pins rather than FAQ watchpoints.

Imported from March ticket 401. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/401-harden-post-migration-spec-runner-and-cli-surface-ratchets
