---
flow: build
priority: 5
---
# Write article fulltext architecture contract doc

Ticket 256 shipped the article fulltext fallback ladder (PMC HTML + opt-in PDF) but the durable architecture corpus does not define resolver priority, accepted source formats, license/PDF/HTML policy, saved-artifact semantics, or failure visibility. Without this contract, future work on article full-text cannot reason about where new fallbacks belong (entity vs source vs renderer) or how errors should surface to users.

Completed under March on 2026-04-22, as March ticket 274. Imported as history when BioMCP
moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/274-write-article-fulltext-architecture-contract-doc
