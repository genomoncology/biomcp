---
base: 368fb02030dcd9ab556638a157e4848f103ebdfb
head: 2f529cb63ddfcb6c656a796eab13b866e104d021
---
What is IN scope: - `src/sources/semantic_scholar.rs` redacted auth-mode helper. - `src/entities/article/{mod.rs,search.rs,backends.rs,enrichment.rs}` or a small adjacent module for source status data flow. - `src/cli/article/dispatch.rs` JSON/debug-plan/markdown rendering of additive source status metadata. - `spec/entity/article.md` and focused Rust/unit tests that pin redacted status behavior.

Imported from March ticket 365. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/365-expose-semantic-scholar-auth-and-degradation-status-in-article-search
