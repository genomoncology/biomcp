---
base: a9ea45b37747ab2078d4a8f6a144f2a50faef40e
head: 90a29d5669f770ee5de1890862712dfe7197f2e5
---
BioMCP encodes real, hard-to-replicate correctness expertise — trial inclusion vs. exclusion discrimination that lifts molecular-eligibility precision from 78.8% to **98.8%** — but only on `--criteria`/`--prior-therapies`/`--progression-on`, not on `--mutation`, the flag a mutation-bearing agent naturally reaches for. On `--mutation` the agent gets raw essie and ~13% of returned trials actually **exclude** the mutation. Encoded correctness that hides behind an obscure flag is, to an agent, not there (P2). Separately, the CTGov trial path re-fetches the same trial detail once per eligibility post-filter.

Imported from March ticket 580. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/580-put-molecular-eligibility-on-the-trial-mutation-path-and-reuse-detail-fetch
