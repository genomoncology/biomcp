---
flow: spike
priority: 8
---
# Spike: Reduce biomcp get gene <symbol> all runtime under 30s budget

`biomcp get gene <symbol> all` runtime is ~44–45s for common genes, which is close to the 60s `--mustmatch-timeout` used by live spec blocks. Any spec that chains two `all` calls (markdown + JSON) exceeds the budget. Ticket 209 repaired the spec layout to dodge the timeout but the underlying latency is unchanged. Reducing runtime both improves agent ergonomics and lets specs go back to exercising both shapes in a single block.

Completed under March on 2026-04-18, as March ticket 225. Imported as history when this
repo moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/225-reduce-biomcp-get-gene-symbol-all-runtime-under-30s-budget
