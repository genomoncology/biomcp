---
base: b032999056040db7a921eb69fb4d58606fca345c
head: 87bc6e4553d3f85bfa71c92687d2257909c85860
---
Spike 244 produced `biomcp_kb_rust_probe`, a Rust crate with JATS, HTML, and PDF-to-Markdown extractors, using `unpdf`, `pdf_oxide`, `readability-rust`, and `html2md`. The crate proved that bounded PDF fallback extraction works (5/6 Rust PDF successes, 8/9 overall across biomedical PDFs, DailyMed labels, and CDC guidelines) and that JATS/HTML paths produce agent-readable Markdown. Before adopting this into `biomcp-cli` to upgrade article fulltext paths, we need an architecture ticket that audits every transitive dependency license, designs the integration into existing fulltext surfaces, and decomposes the adoption into build slices. The vault/frontmatter/Obsidian features from the spike are explicitly deferred.

Imported from March ticket 250. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/250-architecture-adopt-pdf-html-jats-to-markdown-crate-for-article-fulltext
