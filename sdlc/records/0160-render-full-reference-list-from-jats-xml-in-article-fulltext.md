---
base: 15c4e25e5661842f79c32d6fc315454aefc94038
head: f68a2589043cd3b97cf825b60f524548751d21b7
---
The JATS-to-Markdown converter extracts full article body text with proper heading hierarchy, inline formatting, and figure/table captions — but discards the entire bibliography. Inline citation markers like `[1]` and `[^4^]` appear in the body text with no corresponding reference entries. The `references_summary` function counts `<ref>` elements and emits only "87 references cited." instead of rendering the actual references.

Imported from March ticket 160. The range was recovered after the fact, then
corrected by operator review on 2026-08-10 to the main-reachable landed commit
`f68a2589043cd3b97cf825b60f524548751d21b7`. Ticket-owned patches are
byte-identical after excluding the unrelated paths
`src/sources/wikipathways.rs` and all of `spec/06-article.md`; that specification
file differs only by dead `if false` guard wording. The recorded value is
reproducible with
`git diff --binary <base> <head> -- . ':(exclude)src/sources/wikipathways.rs' ':(exclude)spec/06-article.md' | sha256sum`.
The resulting patch SHA-256 is
`fbb28cc86c911f7d526cb331f37fe256540b4823a2a92b960fdafbb458034ec8`.
Both commit objects exist, the recorded base is the landed head's parent, and
the landed head is an ancestor of current main. This note deliberately does
not claim whole-tree equivalence for the excluded paths.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/160-render-full-reference-list-from-jats-xml-in-article-fulltext
