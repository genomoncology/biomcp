---
flow: build
priority: 9
---
# Repair SPEC_SMOKE_ARGS stale line-qualified node IDs and ratchet collectability

`make spec-smoke` fails during pytest collection because `SPEC_SMOKE_ARGS` in the Makefile pins markdown node IDs with literal line-number suffixes that no longer match `spec/06-article.md`:

Completed under March on 2026-04-23, as March ticket 288. Imported as history when this
repo moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/288-repair-spec-smoke-args-stale-line-qualified-node-ids-and-ratchet-collectability
