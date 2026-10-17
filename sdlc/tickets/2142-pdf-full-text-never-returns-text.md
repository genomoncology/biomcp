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

Reproduced on current main before any code (2026-10-16, job `1792206207332-78841` on the
macOS gate host at `b138f6097`):

```
$ biomcp --json --no-cache get article 10615958 fulltext --pdf   # exit 0
"full_text_coverage": {
    "coverage": "unavailable",
    "attempts": [ ..., {
      "provider": {"label": "Semantic Scholar PDF", "source": "Semantic Scholar"},
      "source_kind": "pdf", "coverage": "unavailable", "outcome": "unavailable",
      "cache_state": "bypass", "reason": "source_unavailable" } ] }
"full_text_path": null, "full_text_source": null,
"full_text_note": "Full text unavailable: one or more consulted sources could not be retrieved."
}
```

Cause found before fixing: Semantic Scholar's recorded reply for PMID 10615958 lists
`openAccessPdf.url = https://diabetesjournals.org/diabetes/article-pdf/49/1/114/364646/10615958.pdf`,
and the PDF step's outbound policy allowlists only `pdfs.semanticscholar.org` and
`www.semanticscholar.org`, so the open-PDF link is rejected before any download. The
failing part is the link, not the download and not the text extraction.

## Work

- Reproduce on main first and find the cause: the open-PDF link, the download, or the text extraction.
- Fix the cause, and make each failure name which part failed.

## Acceptance

1. A recorded-reply test downloads a captured open PDF and returns its text; it fails on current main.
2. Each failure reason names the failed part.
3. The changelog carries a bullet for this ticket.
