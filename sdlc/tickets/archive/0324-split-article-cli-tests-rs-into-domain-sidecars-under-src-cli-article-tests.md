---
flow: build
priority: 5
---
# Split article CLI tests.rs into domain sidecars under src/cli/article/tests/

`src/cli/article/tests.rs` is 1,374 lines and has become four distinct test suites in one flat sidecar: help/parse, exact-lookup/suggestions, JSON integration, and filter/ranking checks. The article runtime is already subdivided, but the test ownership is not, which keeps one test file over the cap and makes article regressions harder to localize.

Completed under March on 2026-04-27, as March ticket 324. Imported as history when BioMCP
moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/324-split-article-cli-tests-rs-into-domain-sidecars-under-src-cli-article-tests
