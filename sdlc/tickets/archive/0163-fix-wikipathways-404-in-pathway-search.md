---
flow: build
priority: 7
---
# Fix WikiPathways 404 in pathway search

WikiPathways API is returning HTTP 404 with a GitHub Pages "File not found" HTML page. This causes `search all --counts-only` to emit a WARN-level log line containing hundreds of characters of raw HTML into stderr, and pathway search to report "timed out" even when the real issue is a dead upstream.

Completed under March on 2026-04-10, as March ticket 163. Imported as history when BioMCP
moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/163-fix-wikipathways-404-in-pathway-search
