---
flow: build
priority: 3
deps: [2142, 2147]
---

# 2151: Fetch PDF full text in batch under one limiter

Status: OPEN.

Milestone: 0.9.3

## Outcome

`batch article --sections fulltext --pdf` fetches the PDF fallback for many papers in one process, under the one in-process Semantic Scholar limiter and the existing page and size limits, and the MCP typed article get gains the same `pdf` option.

## Evidence

Asked 2026-10-16 by the research team, who need PDF fallbacks for hundreds of papers.

- `get article ID fulltext --pdf` exists (`src/cli/article/mod.rs:288`); batch refuses `--pdf` (`src/cli/system/batch.rs:371`), and the MCP article schema has no pdf option.
- Rate limiters live in one process. Hundreds of separate CLI calls each start a fresh limiter, so only serial calls keep to Semantic Scholar's rate.

## Work

- Add `--pdf` to batch article for the fulltext section, using the same page and size limits as `get article`, through the shared in-process limiter.
- Add `pdf` to the MCP typed article get.
- Report per article which full-text source was used and why the PDF step failed when it did.

## Acceptance

1. A recorded-reply test of a three-article batch with `--pdf` spaces Semantic Scholar requests at the limiter's interval and returns each article's text or a named failure.
2. An MCP contract test calls the typed get with `pdf`.
3. The changelog carries a bullet for this ticket.
