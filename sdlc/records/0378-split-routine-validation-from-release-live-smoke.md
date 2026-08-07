---
base: 493cff24e74c8e0809147083c525d6e0028d5882
head: a77618f32c81f15bc237fcb364746335436fb2ed
---
What is IN scope: - `Makefile` spec/check/release-smoke targets - `.march/validation-profiles.toml` - `tests/test_validation_profile_contract.py` - `spec/surface/test_parallel_isolation_contract.py` - `spec/README-timings.md` - Architecture/runbook docs that explicitly describe `spec-only`, `release-gate`, or release/live-smoke behavior - `tools/biomcp-ci` invocation only as needed to support the live-smoke lane

Imported from March ticket 378. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/378-split-routine-validation-from-release-live-smoke
