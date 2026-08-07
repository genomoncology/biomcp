---
flow: build
priority: 8
---
# Bug fix — batch --json add _meta contract

`biomcp batch gene BRAF,TP53 --json` returns a bare JSON array with no `_meta`:

Completed under March on 2026-03-27, as March ticket 067. Imported as history when BioMCP
moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/067-bug-batch-json-add-meta-contract
