---
base: 2be05834e8b25f1524e9864ffdc4f355033005f2
head: 1d200551b48d47dbd4b48c794f9abd6bda1d1c50
---
Ticket 130 failed at design-review because it still bundled the internal PubMed hydration/data-shaping work with the public `--source pubmed` CLI cutover. This child isolates the backend contract so the row mapper, metadata fallback rules, and page-fill behavior are settled before any public route is exposed.

Imported from March ticket 140. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/140-pubmed-hydration-foundation-for-article-search
