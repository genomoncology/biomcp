---
flow: build
priority: 9
---
# Add Semantic Scholar article search and directness-first merged ranking

Literature-first questions depend on getting answer-bearing papers near the top of the result set. Europe PMC and PubTator are useful search legs, but they do not always surface the most direct answer first. Semantic Scholar search adds a complementary search leg and impact metadata, but citation count alone is not a good primary ranker. BioMCP should merge the search legs and rank by directness first, with citation signal as secondary evidence.

Completed under March on 2026-03-20, as March ticket 034. Imported as history when BioMCP
moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/034-semantic-scholar-directness-ranking

The landed commit range could not be recovered from git, so no
record accompanies this entry. That is a known gap for the
earliest tickets, not a sign the work is missing.
