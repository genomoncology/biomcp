---
flow: build
priority: 8
---
# Audit spec-pr lane and trim or nightly-move slow headings

`make spec-pr` already drops 20+ live-network headings via `SPEC_PR_DESELECT_ARGS`, but it still takes ~20 minutes during ticket verification. Ticket 183's code-review run spent most of its budget waiting on `make spec-pr`, and the original 183 failure was a timeout in the same gate. Every 60-second heading in the lane is a 60-second tax on every ticket. Profile the lane, identify the outliers, and either trim the assertions, fixture the fan-out, or move the heading to the nightly smoke set.

Completed under March on 2026-04-13, as March ticket 188. Imported as history when BioMCP
moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/188-audit-spec-pr-lane-and-trim-or-nightly-move-slow-headings
