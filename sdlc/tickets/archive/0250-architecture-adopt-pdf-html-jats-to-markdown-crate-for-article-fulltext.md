---
flow: architect
priority: 7
---
# Architecture: adopt PDF/HTML/JATS-to-Markdown crate for article fulltext

Spike 244 produced `biomcp_kb_rust_probe`, a Rust crate with JATS, HTML, and PDF-to-Markdown extractors, using `unpdf`, `pdf_oxide`, `readability-rust`, and `html2md`. The crate proved that bounded PDF fallback extraction works (5/6 Rust PDF successes, 8/9 overall across biomedical PDFs, DailyMed labels, and CDC guidelines) and that JATS/HTML paths produce agent-readable Markdown. Before adopting this into `biomcp-cli` to upgrade article fulltext paths, we need an architecture ticket that audits every transitive dependency license, designs the integration into existing fulltext surfaces, and decomposes the adoption into build slices. The vault/frontmatter/Obsidian features from the spike are explicitly deferred.

Completed under March on 2026-04-20, as March ticket 250. Imported as history when this
repo moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/250-architecture-adopt-pdf-html-jats-to-markdown-crate-for-article-fulltext
