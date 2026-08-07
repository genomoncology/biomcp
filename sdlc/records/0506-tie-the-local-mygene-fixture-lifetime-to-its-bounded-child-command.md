---
base: 3b9f935f188735762db2863416190b97a3e134a8
head: c8bd3f70994c78d42a1011ece5c8d5f233794d07
---
Ticket 503 removed public MyGene.info from `tests/json_error_contract.rs`, but its local fixture has an independent 5-second accept deadline while the BioMCP child command is allowed 10 seconds. Under full nextest load, ticket 504 failed after 2,383 tests passed because the fixture emitted `fixture received no request within 5s` before the scheduled child reached it. A focused rerun passed in 1.5 seconds.

Imported from March ticket 506. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/506-tie-the-local-mygene-fixture-lifetime-to-its-bounded-child-command
