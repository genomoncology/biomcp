---
flow: build
priority: 5
---
# Remove the biomcp suggest command entirely (delete verb + offline regex router, no deprecation)

`biomcp suggest "<question>"` is a 100% offline, zero-backend regex/keyword router over a fixed in-binary catalog of ~15 "playbook" routes (`src/cli/suggest/`, ~1400 lines). It matches by ordered substring-keyword gates: the first route whose hardcoded keywords appear in the question wins. It is brittle in a way that defeats its own purpose:

Completed under March on 2026-07-09, as March ticket 488. Imported as history when BioMCP
moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/488-retire-biomcp-suggest-command-and-its-offline-regex-router-point-agents-at-skill-list
