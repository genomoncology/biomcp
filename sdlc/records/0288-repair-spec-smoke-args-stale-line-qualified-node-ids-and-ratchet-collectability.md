---
base: 9bd36a73ff1acd4406b404621e0b8d516a596c31
head: 452d3b760767a20ea7cae0374299fe62ace210e6
---
`make spec-smoke` fails during pytest collection because `SPEC_SMOKE_ARGS` in the Makefile pins markdown node IDs with literal line-number suffixes that no longer match `spec/06-article.md`:

Imported from March ticket 288. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/288-repair-spec-smoke-args-stale-line-qualified-node-ids-and-ratchet-collectability
