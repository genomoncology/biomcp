---
base: 04d2bfad63a88a30d2ab0eaf874d2ed8dba493a4
head: c54e838c2f73cd38ec1a88182d12569bec6bbf48
---
What is IN scope: - `src/cli/discover.rs` - `src/entities/discover.rs` - `src/cli/disease/dispatch.rs` - `src/entities/disease/search.rs` - `src/entities/disease/fallback.rs` - `src/cli/article/dispatch.rs` - `src/entities/article/planner.rs` tests only as needed to expose pre-execution request-command assertions - Unit tests for normalized disease/discover/article request values and fallback/routing decisions

Imported from March ticket 375. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/375-add-request-command-seams-for-disease-discover-and-article
