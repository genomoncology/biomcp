---
flow: quickfix
priority: 9
---
# Stop the article-graph live canary from stripping the Semantic Scholar key it needs

article-graph-live.md runs through tools/biomcp-ci, which unsets S2_API_KEY, so a credentialed verify page hits the anonymous Semantic Scholar rate limit and fails.

Completed under March on 2026-08-01, as March ticket 636. Imported as history when BioMCP
moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/636-stop-the-article-graph-live-canary-from-stripping-the-semantic-scholar-key-it-needs
