---
flow: build
priority: 8
---
# Fix make test-contracts 7 failures

`make test-contracts` fails 7 of 155+ tests, meaning the repo is not meeting its documented quality bar even though `make check` is green. The failures are in three areas: MCP audit drift expectations, stale doc/Makefile contract assertions, and tracked `.march/` artifacts. The `.march/` failure is addressed by ticket 193; this ticket covers the remaining 6 failures.

Completed under March on 2026-04-15, as March ticket 208. Imported as history when this
repo moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/208-fix-make-test-contracts-7-failures
