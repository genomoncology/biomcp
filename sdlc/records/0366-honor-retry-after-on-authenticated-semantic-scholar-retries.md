---
base: e56ae3aac2208a09b46da0ef4dd1e61b2851cd58
head: 9ce28d03a71c0659236109fd31750fda3f65a3c9
---
What is IN scope: - `src/sources/mod.rs::build_http_client()` retry policy construction. - Focused test coverage for authenticated 429 + `Retry-After` behavior. - Regression coverage that unauthenticated Semantic Scholar shared-pool 429 still surfaces the dedicated-key guidance without broad retry loops.

Imported from March ticket 366. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/366-honor-retry-after-on-authenticated-semantic-scholar-retries
