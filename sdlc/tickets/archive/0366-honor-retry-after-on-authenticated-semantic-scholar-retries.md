---
flow: build
priority: 5
---
# Honor Retry-After on authenticated Semantic Scholar retries

What is IN scope: - `src/sources/mod.rs::build_http_client()` retry policy construction. - Focused test coverage for authenticated 429 + `Retry-After` behavior. - Regression coverage that unauthenticated Semantic Scholar shared-pool 429 still surfaces the dedicated-key guidance without broad retry loops.

Completed under March on 2026-04-30, as March ticket 366. Imported as history when this
repo moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/366-honor-retry-after-on-authenticated-semantic-scholar-retries
