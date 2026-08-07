---
flow: quickfix
priority: 5
---
# Add the missing error count to health JSON

`biomcp --json health` reports `healthy`, `warning`, `excluded`, and `total`, but omits the number of sources in `error`. During the audit it returned 54 healthy, 2 warning, 1 excluded, and 58 total while one row was visibly in error. JSON consumers must infer the missing category by subtraction, even though Markdown already prints the error count.

Completed under March on 2026-07-11, as March ticket 498. Imported as history when BioMCP
moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/498-add-the-missing-error-count-to-health-json
