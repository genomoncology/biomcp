---
flow: build
priority: 5
---
# Retire orphaned ORCID client and superseded dead code paths

Dead and orphaned code is accumulating in the source layer. An orphaned ORCID network client (and its rate-limit wiring) remains in `src/sources/orcid.rs` / `src/sources/orcid/` even though the product boundary is **no ORCID API calls** (ORCID is citation-supplied evidence only) — so this is executable code that contradicts a decided boundary. Separately, superseded parallel code paths and unjustified `#[allow(dead_code)]` suppressions are scattered across entity and render modules, where an internal seam awaiting a ticket is indistinguishable from abandoned code.

Completed under March on 2026-07-17, as March ticket 581. Imported as history when this
repo moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/581-retire-orphaned-orcid-client-and-superseded-dead-code-paths
