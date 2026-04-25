# Superseded: Article Fulltext Design History

> **Status:** Superseded on 2026-04-24.
> **Use instead:** `architecture/functional/article-fulltext.md` (current shipped contract from ticket 274).
> **Why kept:** Historical breadcrumb for the earlier article fulltext migration design.

## Historical Context

This background note preserves the existence of the earlier article fulltext
target-state design that predated the shipped v0.8.22 implementation. It is not
the source of truth for current architecture, resolver order, module ownership,
or user-facing behavior.

## What Shipped Instead

BioMCP already ships the capabilities that the retired design was planning:

- `src/entities/article/fulltext.rs` owns the article fulltext resolver policy.
- The resolver ladder keeps XML first, uses PMC HTML as the default fallback,
  and allows Semantic Scholar PDF only with explicit `--pdf` opt-in.
- Saved fulltext artifacts, Markdown headings, and provenance all use
  source-aware labels and metadata.

## Current References

Use these docs for the live architecture story:

- `architecture/functional/article-fulltext.md` for the current contract and
  module ownership.
- `docs/user-guide/article.md` for user-facing fulltext commands and examples.
- `docs/reference/source-licensing.md` for source terms and reuse guidance.

## Retirement Reason

The old technical-path document was retired because it described shipped
article-fulltext behavior as future work and conflicted with the current
functional architecture. Keeping only this superseded background note avoids two
competing stories about the same surface while preserving the historical trail.
