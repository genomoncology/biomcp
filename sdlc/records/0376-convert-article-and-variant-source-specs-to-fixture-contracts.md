---
base: 497eaa663cb928ebd1df5a1b5a2b80d3ee16159f
head: 5b321a3d42534893dde96cd835e77e79e14d63c1
---
What is IN scope: - `src/entities/article/{backends.rs,query.rs,planner.rs,search.rs}` and related tests - `src/sources/{pubmed.rs,europepmc.rs,pubtator.rs,litsense2.rs,semantic_scholar.rs}` request-plan/status tests - `src/cli/variant/dispatch.rs` tests only where needed to connect existing `VariantSearchPlan` to source plans - `src/entities/variant/{search/mod.rs,normalization.rs}` tests - `src/sources/{myvariant.rs,mutalyzer.rs,variantvalidator.rs}` request-plan/status tests - `spec/entity/article.md`, `spec/entity/variant.md`, and fixtures needed for deterministic executable coverage

Imported from March ticket 376. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/376-convert-article-and-variant-source-specs-to-fixture-contracts
