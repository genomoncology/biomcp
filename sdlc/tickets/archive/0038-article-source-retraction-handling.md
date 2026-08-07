---
flow: build
priority: 10
---
# Fix article-source suppression under default retraction filtering

Default article search currently suppresses PubTator and Semantic Scholar rows whenever `exclude_retracted=true`, because those sources do not provide retraction metadata and BioMCP treats unknown retraction status as retracted. This breaks the intended federated search behavior and hides strong upstream results that the APIs already return.

Completed under March on 2026-03-21, as March ticket 038. Imported as history when BioMCP
moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/038-article-source-retraction-handling

The landed commit range could not be recovered from git, so no
record accompanies this entry. That is a known gap for the
earliest tickets, not a sign the work is missing.
