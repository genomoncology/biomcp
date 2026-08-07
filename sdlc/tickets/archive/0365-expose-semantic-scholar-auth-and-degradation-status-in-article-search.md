---
flow: build
priority: 8
---
# Expose Semantic Scholar auth and degradation status in article search

What is IN scope: - `src/sources/semantic_scholar.rs` redacted auth-mode helper. - `src/entities/article/{mod.rs,search.rs,backends.rs,enrichment.rs}` or a small adjacent module for source status data flow. - `src/cli/article/dispatch.rs` JSON/debug-plan/markdown rendering of additive source status metadata. - `spec/entity/article.md` and focused Rust/unit tests that pin redacted status behavior.

Completed under March on 2026-04-30, as March ticket 365. Imported as history when this
repo moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/365-expose-semantic-scholar-auth-and-degradation-status-in-article-search
