---
flow: quickfix
priority: 5
---
# Tie the local MyGene fixture lifetime to its bounded child command

Ticket 503 removed public MyGene.info from `tests/json_error_contract.rs`, but its local fixture has an independent 5-second accept deadline while the BioMCP child command is allowed 10 seconds. Under full nextest load, ticket 504 failed after 2,383 tests passed because the fixture emitted `fixture received no request within 5s` before the scheduled child reached it. A focused rerun passed in 1.5 seconds.

Completed under March on 2026-07-12, as March ticket 506. Imported as history when this
repo moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/506-tie-the-local-mygene-fixture-lifetime-to-its-bounded-child-command
