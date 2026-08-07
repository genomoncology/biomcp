---
flow: build
priority: 5
---
# Render full reference list from JATS XML in article fulltext

The JATS-to-Markdown converter extracts full article body text with proper heading hierarchy, inline formatting, and figure/table captions — but discards the entire bibliography. Inline citation markers like `[1]` and `[^4^]` appear in the body text with no corresponding reference entries. The `references_summary` function counts `<ref>` elements and emits only "87 references cited." instead of rendering the actual references.

Completed under March on 2026-04-10, as March ticket 160. Imported as history when this
repo moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/160-render-full-reference-list-from-jats-xml-in-article-fulltext
