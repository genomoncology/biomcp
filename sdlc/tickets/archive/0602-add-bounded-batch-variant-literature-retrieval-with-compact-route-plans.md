---
flow: build
priority: 7
---
# Add bounded batch variant-literature retrieval with compact route plans

An agent retrieving literature for seven exact variants had to author a 36-query shell matrix and consumed 414,154 model tokens. BioMCP already has compact article search and `article batch`, but `variant articles` accepts one free-form identifier, exposes no `--debug-plan`, and cannot accept structured transcript/genomic identity for several variants in one request. The missing orchestration surface makes callers duplicate alias expansion, lose provenance, and pay repeatedly for shell/JSON plumbing.

Completed under March on 2026-07-21, as March ticket 602. Imported as history when this
repo moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/602-add-bounded-batch-variant-literature-retrieval-with-compact-route-plans
