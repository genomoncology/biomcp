---
flow: build
priority: 10
---
# Bound remote response body and archive resource consumption

Remote responses are buffered and copied without consistent bounds in the transport/download layer. The shared cached HTTP client applies its body-size limit **after** materializing the response into the cache, so an oversized body is fully buffered before rejection. cBioPortal study archive download and expansion are unbounded (a zip-bomb / oversized-archive class exposure). And after bounded reads the CTGov/PubMed paths still make redundant `.to_vec()` copies of the body. Together these are a resource-consumption and DoS-resistance gap in the layer every source flows through.

Completed under March on 2026-07-16, as March ticket 578. Imported as history when BioMCP
moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/578-bound-remote-response-body-and-archive-resource-consumption
