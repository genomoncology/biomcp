---
flow: build
priority: 8
---
# Put molecular eligibility on the trial mutation path and reuse detail fetch

BioMCP encodes real, hard-to-replicate correctness expertise — trial inclusion vs. exclusion discrimination that lifts molecular-eligibility precision from 78.8% to **98.8%** — but only on `--criteria`/`--prior-therapies`/`--progression-on`, not on `--mutation`, the flag a mutation-bearing agent naturally reaches for. On `--mutation` the agent gets raw essie and ~13% of returned trials actually **exclude** the mutation. Encoded correctness that hides behind an obscure flag is, to an agent, not there (P2). Separately, the CTGov trial path re-fetches the same trial detail once per eligibility post-filter.

Completed under March on 2026-07-17, as March ticket 580. Imported as history when BioMCP
moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/580-put-molecular-eligibility-on-the-trial-mutation-path-and-reuse-detail-fetch
