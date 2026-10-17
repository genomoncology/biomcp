---
flow: build
priority: 3
deps: [2146, 2150]
---

# 2152: Search inside an article's supplements

Status: OPEN.

Milestone: 0.9.4

## Outcome

From a PMID or an asset key, BioMCP extracts the text of an article's xlsx, xls, csv, docx, pdf and zip supplements, searches it for a given string, and returns each hit with an exact locator. It matches text only and interprets nothing.

## Evidence

Asked 2026-10-16 by the research team. Many variant reports appear only in supplement tables, which article search cannot reach today.

- Acceptance examples from the team: BRCA1 F1761I in PMID 30209399 is on Sheet1 row 2780 of its supplement; AXIN2 P245S in PMID 28944238 is on sheet ST2 row 85.

## Work

- Design first: name the command, the extraction library for each format, size and time limits, and where extracted text is cached. Record the design in this ticket before code.
- Return each hit with its locator: worksheet and row for spreadsheets, page for PDFs, paragraph for documents, member path inside archives, plus the source file's checksum.
- Report each file that could not be extracted, with its format and reason, so an empty search can be told from an unread file.

## Acceptance

1. Recorded-file tests find both acceptance examples at the stated locators.
2. A test with an unreadable file reports it with format and reason.
3. The changelog carries a bullet for this ticket.
