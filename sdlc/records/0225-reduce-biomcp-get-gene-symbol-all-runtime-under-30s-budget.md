---
base: 008d61e9bd8b2b0e9086ac2ee320ad7f10f4ab70
head: e8197566203422608fa30d92857e7f1f6f649f43
---
`biomcp get gene <symbol> all` runtime is ~44–45s for common genes, which is close to the 60s `--mustmatch-timeout` used by live spec blocks. Any spec that chains two `all` calls (markdown + JSON) exceeds the budget. Ticket 209 repaired the spec layout to dodge the timeout but the underlying latency is unchanged. Reducing runtime both improves agent ergonomics and lets specs go back to exercising both shapes in a single block.

Imported from March ticket 225. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/225-reduce-biomcp-get-gene-symbol-all-runtime-under-30s-budget
