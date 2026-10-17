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

## Reproduction on main

A gate job on the macOS gate host (`1792234191372-19559`) ran at
`4ce343291` — origin/main `40483dcab` plus this ticket's tests and fixture
logging, no product change — with:

```
cargo build --no-default-features --bin biomcp
./target/debug/biomcp --json batch article 22663011 --mode detail --sections fulltext --pdf
cargo nextest run --no-default-features -E 'test(batch_article_detail_pdf) | test(typed_article_get_maps_the_pdf_option) | test(typed_article_get_accepts_pdf_fulltext)'
```

- The gap command exits 2 before any provider request:
  `Invalid argument: error: unexpected argument '--pdf' found` — clap knows no
  `--pdf` on `batch`, so the only way to fetch PDF fallbacks for many papers is
  hundreds of separate `get article` processes, each with a fresh limiter.
- All four recorded tests FAILED on main (exit 100):
  `batch_article_detail_pdf_fetches_three_papers_under_one_limiter` (exit 2,
  clap refusal), `batch_article_detail_pdf_names_the_fulltext_source_each_article_used`
  (exit 2), `typed_article_get_accepts_pdf_fulltext_over_stdio` (typed get
  rejects the field: `unknown article get field: pdf`), and
  `typed_article_get_maps_the_pdf_option_to_the_cli_flag` (no `pdf` property in
  the typed schema, no `--pdf` in the mapped argv).

## Work

- Add `--pdf` to batch article for the fulltext section, using the same page and size limits as `get article`, through the shared in-process limiter.
- Add `pdf` to the MCP typed article get.
- Report per article which full-text source was used and why the PDF step failed when it did.

## Acceptance

1. A recorded-reply test of a three-article batch with `--pdf` spaces Semantic Scholar requests at the limiter's interval and returns each article's text or a named failure.
2. An MCP contract test calls the typed get with `pdf`.
3. The changelog carries a bullet for this ticket.
