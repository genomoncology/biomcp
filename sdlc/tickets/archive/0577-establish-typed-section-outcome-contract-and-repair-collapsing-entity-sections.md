---
flow: build
priority: 10
---
# Establish typed section-outcome contract and repair collapsing entity sections

Optional entity sections convert failed upstream retrieval into normal empty biomedical collections. QuickGO/STRING map to `Some([])` and render as healthy empty gene sections; OpenFDA Drugs@FDA failure renders `approvals: []`; a missing/unreadable CVX bundle makes a CVX-dependent brand (e.g. Gardasil) return `query_not_vaccine`; protein (InterPro/STRING/ComplexPortal) and Reactome sections render empty on failure. In every case the command exits 0, emits no in-band degradation, and still names the provider in `_meta.section_sources`, so a JSON/MCP caller cannot distinguish a confirmed zero from failed retrieval. These are release-significant false biomedical negatives. Reproduced at 0.8.25 (`e56630be`): `BRAF go:[]` / `interactions:[]` on unreachable QuickGO/STRING, `pembrolizumab approvals:[]` on unreachable OpenFDA, `Gardasil query_not_vaccine` with missing `BIOMCP_CVX_DIR`. The same `Some(Vec::new())` fallback is duplicated across sequential and `parallel-top` orchestration, so the fix must be one shared model, not five point patches.

Completed under March on 2026-07-16, as March ticket 577. Imported as history when BioMCP
moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/577-establish-typed-section-outcome-contract-and-repair-collapsing-entity-sections
