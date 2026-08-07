---
base: 7e737dbd45d766faf3dcac07960a1ac274597c81
head: 576f1799522202f52def527f23b39b1e43608266
---
Optional entity sections convert failed upstream retrieval into normal empty biomedical collections. QuickGO/STRING map to `Some([])` and render as healthy empty gene sections; OpenFDA Drugs@FDA failure renders `approvals: []`; a missing/unreadable CVX bundle makes a CVX-dependent brand (e.g. Gardasil) return `query_not_vaccine`; protein (InterPro/STRING/ComplexPortal) and Reactome sections render empty on failure. In every case the command exits 0, emits no in-band degradation, and still names the provider in `_meta.section_sources`, so a JSON/MCP caller cannot distinguish a confirmed zero from failed retrieval. These are release-significant false biomedical negatives. Reproduced at 0.8.25 (`e56630be`): `BRAF go:[]` / `interactions:[]` on unreachable QuickGO/STRING, `pembrolizumab approvals:[]` on unreachable OpenFDA, `Gardasil query_not_vaccine` with missing `BIOMCP_CVX_DIR`. The same `Some(Vec::new())` fallback is duplicated across sequential and `parallel-top` orchestration, so the fix must be one shared model, not five point patches.

Imported from March ticket 577. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/577-establish-typed-section-outcome-contract-and-repair-collapsing-entity-sections
