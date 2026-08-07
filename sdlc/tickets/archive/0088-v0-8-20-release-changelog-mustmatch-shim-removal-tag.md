---
flow: build
priority: 7
---
# v0.8.20 release — changelog, mustmatch shim removal, tag

70+ commits have landed since v0.8.19 (tickets 075-087) covering product fixes, cross-entity links, quality ratchet, architecture docs, and repo cleanup. The quality ratchet script (`tools/check-quality-ratchet.sh`) inlines mustmatch lint logic because `mustmatch lint` wasn't on PyPI until 0.0.4, which was just published. This release removes that shim, updates the changelog, and tags.

Completed under March on 2026-03-30, as March ticket 088. Imported as history when BioMCP
moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/088-v0-8-20-release-changelog-mustmatch-shim-removal-tag
