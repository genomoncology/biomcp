---
flow: build
priority: 7
---
# Fix documentation drift: README sources, CHANGELOG WHO, root help count

The README source table, CHANGELOG, and root help text have not been updated for the v0.8.17-0.8.20 data source expansion batch. The README "Entities and sources" table is missing SEER Explorer, NIH Reporter, DisGeNET, and WikiPathways. The CHANGELOG 0.8.20 section omits WHO Prequalification despite the feature shipping. The root help text in `src/cli/types.rs:12` hardcodes "15 biomedical sources" when the health output now reports 51 endpoints.

Completed under March on 2026-04-15, as March ticket 211. Imported as history when this
repo moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/211-fix-documentation-drift-readme-sources-changelog-who-root-help-count
