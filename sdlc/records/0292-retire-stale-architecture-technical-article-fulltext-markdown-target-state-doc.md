---
base: 6e3a83dfebc835c2856aafe685e9ce30acb010f9
head: 563970a47169220e71a4cb4ee84ccfa34306a1cd
---
`architecture/technical/article-fulltext-markdown.md` is stale and now contradicts shipped code. It claims the "current implementation remains JATS-only" and frames `src/entities/article/fulltext.rs`, HTML fallback, PDF fallback, source-aware cache keys, and `--pdf` as target-state work.

Imported from March ticket 292. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/292-retire-stale-architecture-technical-article-fulltext-markdown-target-state-doc
