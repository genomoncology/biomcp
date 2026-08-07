---
base: 57b688bcbd4bcf159e90b728278f37ce14e88415
head: 1d0fba9added0ebaa97d570bfb264796bec482b8
---
`spec/surface/discover.md::Disease-Specific Symptom Phrases Stay Clinically Modest` lost two assertions during ticket 310 verify because `tools/biomcp-ci` strips `UMLS_API_KEY` to keep CI off live UMLS. That left one shipped assertion (the `phenotypes` command suggestion) and a coverage gap for what the surface actually produces in the UMLS-unavailable lane (the dominant CI/operator path). The 327 review flagged this as a behavioral coverage gap, not a runtime defect.

Imported from March ticket 338. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/338-rewrite-umls-stripped-discover-spec-assertions-to-cover-degraded-mode-behavior
