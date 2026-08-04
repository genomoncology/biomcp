# Code Review — Ticket 670

## Scope reviewed

- Read `AGENTS.md`, ticket/design artifacts, contract red check, and code log.
- Reviewed the full `main..HEAD` diff, including CPIC receipts, decoder tests, fixture lifecycle, runner routing, routine/live registries, specs, and lane documentation.
- Compared all final-design acceptance criteria and proof-matrix entries to the landed diff.

## Audit results

- **Design completeness:** all named implementation, receipt, fixture, routing, docs, and test work landed.
- **Spec traceability:** all five `spec/entity/pgx.md` proof-matrix entries landed. The scoped `git diff main..HEAD -- 'spec/*'` contains no invented, removed, or silently relaxed shipped assertion. The ellipsis edits preserve the designed anchors and correct mustmatch mechanics.
- **Edit discipline:** 480 added / 48 removed lines is proportionate to six raw receipt-backed captures plus a bounded loopback fixture and required routing/tests. No excess runtime edits or unlogged adjacent fixes found.
- **Security/quality:** fixture binds only loopback, validates fixed route/query shapes, uses owned process-group cleanup, and serves repository-controlled bytes. Receipts contain public unsigned URLs and their hashes verify. No duplicated production HTTP/decoder implementation or resource-cleanup issue found.

## Fix applied

The Tier 3 gene-pair decoder test retained the obsolete fixture header `Content-Range: 0-0/12` despite decoding the new 79-row receipt-backed capture. It consequently asserted a fabricated total of 12. The runner fixture had the same inconsistent `0-14/*` header while serving all 79 raw rows.

Repaired both to use `Content-Range: 0-78/*`; the decoder test now asserts the correct unknown total (`None`) and 79 decoded rows. This restores a valid pagination contract without changing shipped specs or runtime behavior.

## Validation

- `cargo nextest run --no-default-features cpic` — pass (12 tests)
- `uv run tools/check-source-capture-receipts.py --root testdata/sources` — pass
- `make lint` — pass
- `make test` — pass (439 Python contracts; native tests and strict MkDocs)
- `make spec` — pass (218 Markdown passed, 2 skipped; 90/3 article lane; 7 section outcomes; 31 parallel-isolation tests; 10 static specs)

## Residual concerns

None within ticket scope. Receipt-backed raw JSON intentionally retains provider whitespace; it is hash-verified and must not be reformatted.

## Defect Register

| # | Category | Lintable | Description |
|---|----------|----------|-------------|
| 1 | weak-assertion | no | `src/sources/cpic/tests/parsing.rs` carried a legacy, fabricated 12-row pagination total for a 79-row receipt-backed capture; corrected the test and loopback response header to `0-78/*`. |
