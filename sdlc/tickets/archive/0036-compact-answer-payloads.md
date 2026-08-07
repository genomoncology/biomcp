---
flow: build
priority: 9
---
# Add compact answer-bearing payloads for typed retrieval

Several typed BioMCP paths already retrieve enough evidence to answer the question, but the result shape still forces the agent to read tables and infer the answer manually. Adding compact answer-bearing fields for dates, percentages, and disease/variant pivots would reduce tool calls without changing the typed retrieval contract or inventing unsupported synthesis.

Completed under March on 2026-03-20, as March ticket 036. Imported as history when BioMCP
moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/036-compact-answer-payloads

The landed commit range could not be recovered from git, so no
record accompanies this entry. That is a known gap for the
earliest tickets, not a sign the work is missing.
