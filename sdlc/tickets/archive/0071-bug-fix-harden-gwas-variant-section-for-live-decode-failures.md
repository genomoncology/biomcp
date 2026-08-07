---
flow: build
priority: 7
---
# Bug fix — harden GWAS variant section for live decode failures

`make spec-pr` currently fails the `spec/03-variant.md::GWAS Supporting PMIDs` heading. The underlying command `biomcp --json get variant rs7903146 gwas` fails with `Error: HTTP request failed: error decoding response body`. The GWAS REST API is returning a response that cannot be decoded by the current deserializer.

Completed under March on 2026-03-27, as March ticket 071. Imported as history when BioMCP
moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/071-bug-fix-harden-gwas-variant-section-for-live-decode-failures
