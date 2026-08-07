---
flow: build
priority: 8
---
# Convert article and variant source specs to fixture contracts

What is IN scope: - `src/entities/article/{backends.rs,query.rs,planner.rs,search.rs}` and related tests - `src/sources/{pubmed.rs,europepmc.rs,pubtator.rs,litsense2.rs,semantic_scholar.rs}` request-plan/status tests - `src/cli/variant/dispatch.rs` tests only where needed to connect existing `VariantSearchPlan` to source plans - `src/entities/variant/{search/mod.rs,normalization.rs}` tests - `src/sources/{myvariant.rs,mutalyzer.rs,variantvalidator.rs}` request-plan/status tests - `spec/entity/article.md`, `spec/entity/variant.md`, and fixtures needed for deterministic executable coverage

Completed under March on 2026-05-23, as March ticket 376. Imported as history when BioMCP
moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/376-convert-article-and-variant-source-specs-to-fixture-contracts
