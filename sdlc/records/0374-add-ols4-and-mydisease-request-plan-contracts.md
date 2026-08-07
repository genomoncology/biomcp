---
base: fe3f9ce79c61fa7b228bb2fcf27fb4b67c4abf8a
head: a3d52916840c98299cb083394a4dcb1fc7cd11de
---
What is IN scope: - `src/sources/ols4.rs` - `src/sources/mydisease.rs` - Source-local tests for OLS4/MyDisease request plans and response/status mapping - Minimal disease/discover fixture or unit coverage needed to restore `Synonym Rescue` and `MEF2 relational query` as deterministic contracts - `spec/entity/disease.md`, `spec/surface/discover.md`, `spec/README-timings.md`, and `spec/surface/test_parallel_isolation_contract.py` only if the quarantine text or OLS4 partition ratchet must change to reflect replacement coverage

Imported from March ticket 374. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/374-add-ols4-and-mydisease-request-plan-contracts
