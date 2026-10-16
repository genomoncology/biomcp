---
flow: build
priority: 3
deps: []
---

# 2142: Return text from the PDF full-text step

Status: OPEN.

Milestone: 0.9.3

## Outcome

`get article ... fulltext --pdf` returns the PDF's text when Semantic Scholar lists an open PDF, and names the failure when it cannot.

## Evidence

Reported 2026-10-16 by a literature probe of 0.9.2.

- `biomcp --json --no-cache get article 10615958 fulltext --pdf` reports `source_unavailable` at the PDF step.
- All 12 probe papers that reached the PDF step got no text: 7 `source_unavailable`, 5 `no_content`.

## Work

- Reproduce on main first and find the cause: the open-PDF link, the download, or the text extraction.
- Fix the cause, and make each failure name which part failed.

## Acceptance

1. A recorded-reply test downloads a captured open PDF and returns its text; it fails on current main.
2. Each failure reason names the failed part.
3. The changelog carries a bullet for this ticket.
