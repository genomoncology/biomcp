---
base: 1f1c39035db59de11a9322a31eb9047ff1de8b04
head: e5b678f119c8b34fe3ab75f2ee41b2f7351daba2
---
`make test-contracts` fails 7 of 155+ tests, meaning the repo is not meeting its documented quality bar even though `make check` is green. The failures are in three areas: MCP audit drift expectations, stale doc/Makefile contract assertions, and tracked `.march/` artifacts. The `.march/` failure is addressed by ticket 193; this ticket covers the remaining 6 failures.

Imported from March ticket 208. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/208-fix-make-test-contracts-7-failures
