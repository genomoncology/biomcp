---
base: 15c4e25e5661842f79c32d6fc315454aefc94038
head: f6e634c6b480942ba71b81215ebf1843a3d5384f
---
The JATS-to-Markdown converter extracts full article body text with proper heading hierarchy, inline formatting, and figure/table captions — but discards the entire bibliography. Inline citation markers like `[1]` and `[^4^]` appear in the body text with no corresponding reference entries. The `references_summary` function counts `<ref>` elements and emits only "87 references cited." instead of rendering the actual references.

Imported from March ticket 160. The commit range was
recovered after the fact (branch march/160-render-full-reference-list-from-jats-xml-in-article-fulltext named for the ticket slug), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/160-render-full-reference-list-from-jats-xml-in-article-fulltext
