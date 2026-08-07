---
flow: build
priority: 7
---
# Sweep seven removed short-literal spec-v2 assertions with stronger replacements

Seven mustmatch assertions in the spec-v2 corpus were authored as short literals (under the 10-char ratchet) and got removed in verify because they were both syntactically below threshold and semantically too weak (e.g. `mustmatch like "suggest"` would pass on the substring "suggested"). The runtime contracts they were meant to protect are now uncovered.

Completed under March on 2026-04-25, as March ticket 308. Imported as history when BioMCP
moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/308-sweep-seven-removed-short-literal-spec-v2-assertions-with-stronger-replacements
