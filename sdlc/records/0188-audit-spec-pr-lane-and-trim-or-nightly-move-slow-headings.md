---
base: 5947f21c7bb3a65855f96f3f317aeb20f5918370
head: 23422286cba66a46025acc8642a3bd7be2e617f7
---
`make spec-pr` already drops 20+ live-network headings via `SPEC_PR_DESELECT_ARGS`, but it still takes ~20 minutes during ticket verification. Ticket 183's code-review run spent most of its budget waiting on `make spec-pr`, and the original 183 failure was a timeout in the same gate. Every 60-second heading in the lane is a 60-second tax on every ticket. Profile the lane, identify the outliers, and either trim the assertions, fixture the fan-out, or move the heading to the nightly smoke set.

Imported from March ticket 188. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/188-audit-spec-pr-lane-and-trim-or-nightly-move-slow-headings
