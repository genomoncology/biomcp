---
base: 4d6b771ecf78dec459f2187f8b2b1d4ec8dfa3dd
head: 9bbdef70a4fe4dcd8ff68ff09214fd2612c1cf0c
---
The README source table, CHANGELOG, and root help text have not been updated for the v0.8.17-0.8.20 data source expansion batch. The README "Entities and sources" table is missing SEER Explorer, NIH Reporter, DisGeNET, and WikiPathways. The CHANGELOG 0.8.20 section omits WHO Prequalification despite the feature shipping. The root help text in `src/cli/types.rs:12` hardcodes "15 biomedical sources" when the health output now reports 51 endpoints.

Imported from March ticket 211. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/211-fix-documentation-drift-readme-sources-changelog-who-root-help-count
