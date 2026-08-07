---
flow: quickfix
priority: 6
---
# Retire stale architecture technical article-fulltext-markdown target-state doc

`architecture/technical/article-fulltext-markdown.md` is stale and now contradicts shipped code. It claims the "current implementation remains JATS-only" and frames `src/entities/article/fulltext.rs`, HTML fallback, PDF fallback, source-aware cache keys, and `--pdf` as target-state work.

Completed under March on 2026-04-24, as March ticket 292. Imported as history when BioMCP
moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/292-retire-stale-architecture-technical-article-fulltext-markdown-target-state-doc
