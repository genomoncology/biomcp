---
flow: build
priority: 9
---
# Fix search all --counts-only JSON contract and performance

`biomcp search all --counts-only` advertises a counts-first orientation mode, but the implementation still executes full per-section fetches and the JSON surface returns the full `results` payload. This is both a correctness defect (the JSON response does not match the semantic promise of `--counts-only`) and a performance defect (the hot path pays full fan-out cost even when the caller explicitly asked for a lightweight summary). Agent callers using `--counts-only` for orientation get inflated token volume with no benefit.

Completed under March on 2026-04-14, as March ticket 206. Imported as history when BioMCP
moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/206-fix-search-all-counts-only-json-contract-and-performance
