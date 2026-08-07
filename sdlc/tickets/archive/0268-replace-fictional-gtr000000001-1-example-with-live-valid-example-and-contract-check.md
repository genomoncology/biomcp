---
flow: build
priority: 8
---
# Replace fictional GTR000000001.1 example with live-valid example and contract check

`GTR000000001.1` is used as the example diagnostic accession in seven locations across `README.md` and `docs/user-guide/diagnostic.md`, but that accession does not exist in the live GTR bundle — running `biomcp get diagnostic GTR000000001.1` returns "not found". First-time users following docs hit a dead end. No contract check protects public example accessions from drifting.

Completed under March on 2026-04-21, as March ticket 268. Imported as history when BioMCP
moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/268-replace-fictional-gtr000000001-1-example-with-live-valid-example-and-contract-check
