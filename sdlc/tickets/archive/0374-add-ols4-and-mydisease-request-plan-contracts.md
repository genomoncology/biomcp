---
flow: build
priority: 10
---
# Add OLS4 and MyDisease request-plan contracts

What is IN scope: - `src/sources/ols4.rs` - `src/sources/mydisease.rs` - Source-local tests for OLS4/MyDisease request plans and response/status mapping - Minimal disease/discover fixture or unit coverage needed to restore `Synonym Rescue` and `MEF2 relational query` as deterministic contracts - `spec/entity/disease.md`, `spec/surface/discover.md`, `spec/README-timings.md`, and `spec/surface/test_parallel_isolation_contract.py` only if the quarantine text or OLS4 partition ratchet must change to reflect replacement coverage

Completed under March on 2026-05-23, as March ticket 374. Imported as history when this
repo moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/374-add-ols4-and-mydisease-request-plan-contracts
