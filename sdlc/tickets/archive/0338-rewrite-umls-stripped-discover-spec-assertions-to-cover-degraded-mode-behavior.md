---
flow: quickfix
priority: 4
---
# Rewrite UMLS-stripped discover spec assertions to cover degraded-mode behavior

`spec/surface/discover.md::Disease-Specific Symptom Phrases Stay Clinically Modest` lost two assertions during ticket 310 verify because `tools/biomcp-ci` strips `UMLS_API_KEY` to keep CI off live UMLS. That left one shipped assertion (the `phenotypes` command suggestion) and a coverage gap for what the surface actually produces in the UMLS-unavailable lane (the dominant CI/operator path). The 327 review flagged this as a behavioral coverage gap, not a runtime defect.

Completed under March on 2026-04-29, as March ticket 338. Imported as history when BioMCP
moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/338-rewrite-umls-stripped-discover-spec-assertions-to-cover-degraded-mode-behavior
