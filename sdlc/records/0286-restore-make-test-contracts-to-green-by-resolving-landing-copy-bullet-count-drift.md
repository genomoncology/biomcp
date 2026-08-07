---
base: 0ae5981ad38648894e0b453ecc29fb22c8cde146
head: 236d30d951cca3edff4f74ebe85d9de67dc13512
---
`make test-contracts` is red before the v0.8.22 cut. Two landing-copy contract assertions fail:

Imported from March ticket 286. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/286-restore-make-test-contracts-to-green-by-resolving-landing-copy-bullet-count-drift
