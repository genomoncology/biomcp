---
flow: build
priority: 3
deps: [2146, 2150]
---

# 2152: Search inside an article's supplements

Status: OPEN.

Milestone: 0.9.3

## Outcome

From a PMID or an asset key, BioMCP extracts the text of an article's xlsx, xls, csv, docx, pdf and zip supplements, searches it for a given string, and returns each hit with an exact locator. It matches text only and interprets nothing.

## Evidence

Asked 2026-10-16 by the research team. Many variant reports appear only in supplement tables, which article search cannot reach today.

- Acceptance examples from the team: BRCA1 F1761I in PMID 30209399 is on Sheet1 row 2780 of its supplement; AXIN2 P245S in PMID 28944238 is on sheet ST2 row 85.

## Work

- Design first: name the command, the extraction library for each format, size and time limits, and where extracted text is cached. Record the design in this ticket before code.
- Return each hit with its locator: worksheet and row for spreadsheets, page for PDFs, paragraph for documents, member path inside archives, plus the source file's checksum.
- Report each file that could not be extracted, with its format and reason, so an empty search can be told from an unread file.

## Design (recorded 2026-10-17, before code)

Command: `biomcp --json article supplement-search <id> <needle> [--asset <asset-key>]`.
`<id>` accepts a PMID, PMCID, or DOI. Without `--asset` the command searches
every retrievable manifest asset whose lowercased file extension is one of
the six supplement formats; `--asset` restricts the search to one asset by
asset key or unambiguous file name. Output is JSON-only, like the assets
manifest. The match is an exact case-sensitive substring in the extracted
text: no folding, no word boundaries, no interpretation.

Extraction library per format:

| Format | Library | Block and locator |
| --- | --- | --- |
| xlsx | `calamine` 0.36 (new dependency; the only maintained Rust reader covering OOXML workbooks and binary BIFF `.xls`) | each worksheet row; locator `worksheet` + `row` (1-based Excel row) |
| xls | `calamine` 0.36 | same as xlsx |
| csv | existing `csv` crate | each record; locator `row` (1-based, header is row 1) |
| docx | existing `zip` + `roxmltree` over `word/document.xml` (a docx is a deflate zip; no new dependency) | each `w:p` element in order; locator `paragraph` (1-based) |
| pdf | existing `unpdf` 0.4.5, rendered one page at a time | each page; locator `page` (1-based) |
| zip | existing `zip` crate; each member with a searchable extension recurses (one nested archive level allowed) | locator carries the `member` path plus the member format's own fields |

Row text joins cell values with a tab; paragraph text joins `w:t` runs.

Limits: 64 MiB per source file and 64 MiB decompressed per archive member
with 256 MiB aggregate per source file (over the limit: unextractable,
"extraction size limit exceeded"); 20 s wall clock per source file (over:
unextractable, "extraction time limit exceeded", not cached); 500 PDF pages
per file (later pages unsearched, file row marked truncated); at most 200
hits reported with `hits_truncated` telling the caller to narrow the needle.

Cache: extracted text lands in
`<cache>/sessions/article-supplement-text-v1/<sha256-of-source-file>.json`,
schema 1, keyed by the source file's bytes so a changed supplement writes a
new key and no TTL applies. Entry cap 16 MiB; nothing is written under
`--no-cache`; time-limit failures are never cached.

Envelope: `{article_id, pmid?, pmcid?, query, files[], hits[], hits_truncated}`.
Each `files[]` row carries `asset_key`, `filename`, `format`, `sha256`, and
`outcome` `searched` | `empty_text` (parsed, zero blocks) | `unextractable`
(reason strings above, plus "unsupported format" and per-format parse
failures). Each hit carries the file identity, the source file's `sha256`,
structured locator fields (`member?`, `worksheet?`, `row?`, `page?`,
`paragraph?`), a rendered `locator` string (for example `Sheet1 row 2780`,
`member 'supplement.xlsx' ST2 row 85`), and a `context` excerpt of the
matched block capped at 400 bytes. Figure images are not searched: they are
not supplements.

## Acceptance

1. Recorded-file tests find both acceptance examples at the stated locators.
2. A test with an unreadable file reports it with format and reason.
3. The changelog carries a bullet for this ticket.
