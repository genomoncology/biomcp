---
flow: build
priority: 2
---

# 2152: Search inside an article's supplements

Status: COMPLETE.

Milestone: 0.9.3

Depends on: 2146
Depends on: 2150
Depends on: 2151

Reviews: revision 5a609b5, accept

Reviews: revision 44c3fd5, accept

Reviews: revision c0dc241, accept

Reviews: revision aadfd73, accept

## Outcome

From an article identifier, with an optional asset restriction, BioMCP extracts the text of an article's xlsx, xls, csv, docx, pdf and zip supplements, searches it for a given string, and returns each hit with an exact locator. It matches text only and interprets nothing.

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

## Changes and proof

- Starts from: the existing pushed implementation at `9f2872e2d`; preserve its approved command and complete design above rather than rebuilding it.
- Changes: claims `.gitattributes` `CHANGELOG.md` `Cargo.lock` `Cargo.toml` `deny.toml` `docs/user-guide/article.md` `docs/user-guide/supplementary-materials.md` `spec/entity/article.md` `spec/fixtures/setup-article-fulltext-source-fixture.sh` `src/main.rs` `src/lib.rs` `src/cli/article/dispatch.rs` `src/cli/article/mod.rs` `src/cli/list/literature.rs` `src/cli/response_contract.rs` `src/entities/article/assets.rs` `src/entities/article/mod.rs` `src/entities/article/supplement_search.rs` `src/mcp/shell.rs` `src/transform/article.rs` `src/transform/article/supplement_text.rs` `src/transform/article/supplement_text/tests.rs` `tests/article_asset_terminal.rs` `tests/article_transform_structure.rs` `tests/fixtures/article/supplements/28944238-axin2-st2.xlsx` `tests/fixtures/article/supplements/30209399-brca1-sheet1.xlsx` `tests/fixtures/article/supplements/legacy-supplement.xls` `tests/fixtures/article/supplements/method-notes.docx` `tests/fixtures/article/supplements/supplement-notes.pdf` `tests/test_binary_asset_gitattributes_contract.py` `tests/test_source_package_boundary.py`. Request a claim expansion before changing further files.
- Keeps: exact substring matching without clinical interpretation; an article identifier is always required, and `--asset` only restricts that article's assets. Preserve the stated formats, locators, refusals, cache policy and all limits.
- Proof: before changes, reproduce the remaining defects against current main or the adopted branch as applicable and record the exact entry-point commands. Through `biomcp --json article supplement-search`, execute the two recorded acceptance examples below, malformed/unreadable files, Unicode context bounded to 400 bytes, shared aggregate expansion budgets across nested members and workbook/document parsing, exactly one nested archive level, and a time-limited extraction whose worker actually stops and writes no cache. Check identical bytes presented with different formats cannot reuse incompatible cached parsing. Run affected outside-in cases and existing focused parser checks through the build job; expensive stress cases belong in the explicit long suite.
- Required repairs: share one 256 MiB aggregate budget across every extraction layer; bound workbook processing and DOCX XML reads; permit one nested archive rather than two; enforce the twenty-second wall limit beyond merely abandoning a blocking task; cap context by bytes at character boundaries; validate format compatibility on cache reads while preserving content-addressed paths.
- Defers: a broad fixture-identifier/scenario refactor to 2163. Fix only fixture collisions blocking this contracted behavior.

## Implementation brief

PM decision needed: Restore the build job's compiler wrapper: both unset and zero `CARGO_INCREMENTAL` make its version probe refuse before compilation. The coordinator authorized one temporary `RUSTC_WRAPPER=""` correctness check on the existing warm target; this changes no persistent configuration and supports no performance claim.

Preserve the adopted article-scoped, exact case-sensitive six-format search and its recorded locators and response envelope. Repair the shared extraction budgets, byte-bounded context and cache-format identity. Enforce the extraction timeout by killing and reaping a private worker process. The additional `src/main.rs` and `src/lib.rs` claims expose only the private worker entry point; the user command stays as designed.

Before product edits, current-main build job `1792241016358-72714` on `078ce756d` ran `cargo run --no-default-features -- --json article supplement-search 30209399 F1761I` and exited 2 with JSON error `invalid_argument`, `unrecognized subcommand 'supplement-search'`. Focused extraction-boundary regressions were committed locally at `e4732a000` before implementation; the review-ready job will demonstrate those checks against the adopted production code and then the repair.

## Acceptance

1. Recorded-file tests find both acceptance examples at the stated locators.
2. A test with an unreadable file reports it with format and reason.
3. The changelog carries a bullet for this ticket.

## Integration review

Fresh review accepts dc3a756a4b614641370097a34d1c56cce9d5c579 merged with main beb08a9f3. Supplement parser, search and owned behavioral tests are unchanged. Retain main’s package-membership test without the retired manual file count; preserve stable metadata, batch PDF and actual limiter proof, typed MCP behavior, HTTP diagnostics and current cache behavior. The isolated cache candidate is not restored. Reuse parser/CLI job 1792242191704-62560, empty-ZIP check 1792242437128-73123, OOXML format fix 4f0612e03 and correctness green 1792243006443-85094 (31 cases and formatting). That historical compiler-wrapper exception supports correctness only; the merged landing gate still uses the normal configured wrapper. Land separately to unblock the dependent final public-text cleanup while the cache dependency decision remains pending.

Gate 1792260718966-18437 compiled with the normal wrapper and passed 4290 Rust cases before a helper-name convention failure: the new local cache_path function matches the retired-helper scanner. The source child-stdio guard also fails to recognize the worker constructor setup. Correct these two private worker helper declarations while preserving bounded extraction, stdin EOF, explicitly piped output and stderr handling; retain the guards rather than weakening them. The independent quality wrapper failure reads temporary fulltext fixture lock files during concurrent cleanup and belongs to its own small fixture-output repair. No full-gate rerun until these concrete causes are corrected.

Fresh review accepts worker correction caec06da38ed8ae94e446397d85e37e2541a64aa. Job 1792261149162-54431 passed 13 child-stdio guard cases, formatting, the legacy helper guard, the existing timeout kill/reap case and all-target routine Clippy with the normal wrapper. Both constructors now expose the same explicit stream configuration to the guard; the runner preserves EOF closure, kill-on-drop, timeout kill and reap. The content-addressed helper has an unambiguous owned name. The independent fixture-output race is repaired by 2177, whose reviewed source will join this candidate before its gate.

Shared candidate carries accepted fixture repair 2177 at 7e0ee102f, with its individual review and controlled real-helper red/green proof. Merge changed only the fixture allocator, its regression, changelog and record; no implementation conflict required resolution. Batch members are 2152 and 2177. The final measurement depends on the repair. Run one candidate gate and one full check on actual pushed main. The cache construction developer proceeds independently on its approved dependency batch.
