---
flow: quickfix
priority: 10
---
# Fix figshare URL parser version-segment article-id bug

This is the exact retrieval path that 398 added and 407 is measuring: for variant classification, the decisive per-variant functional datum often lives **only** in an AACR figshare supplement (the abstract/full text doesn't carry it). One worked case:

Completed under March on 2026-06-08, as March ticket 408. Imported as history when BioMCP
moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/408-fix-figshare-url-parser-version-segment-article-id-bug
