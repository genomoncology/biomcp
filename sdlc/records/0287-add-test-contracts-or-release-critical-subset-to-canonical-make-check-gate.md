---
base: 41dd4126ae2d0ae9fcf0bf1bb013119465fe2caa
head: bbdd5160bdc38ca0c139e60ca49956d65480bea4
---
`make check` passed on the v0.8.22 release-readiness tree while `make test-contracts` was red on 2 landing-copy assertions. The canonical local gate cannot be green while the public contract suite is red, or the next release will ship the same class of drift.

Imported from March ticket 287. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/287-add-test-contracts-or-release-critical-subset-to-canonical-make-check-gate
