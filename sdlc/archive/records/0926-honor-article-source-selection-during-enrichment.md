---
base: 91634748
head: 16d630f8
---

Article searches now carry an explicit candidate-source and enrichment-source
plan through JSON, Markdown, and debug routing. Selecting one provider performs
only that provider's candidate request; it no longer silently calls Semantic
Scholar or another enrichment source.

A strict local Semantic Scholar fixture proves single execution with no hidden
enrichment calls. Focused planner, enrichment, CLI, Markdown, output-footprint,
and quality-ratchet checks passed within the authorized source ceiling.
