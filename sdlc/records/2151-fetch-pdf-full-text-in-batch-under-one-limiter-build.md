# 2151 build record — fetch PDF full text in batch under one limiter

## Report being fixed

The research team needs PDF fallbacks for hundreds of papers. `get article <id>
fulltext --pdf` exists, but `batch article` refused `--pdf` at clap parsing and
the typed MCP article get had no `pdf` field, so the only route was hundreds of
separate CLI calls — each process starting a fresh Semantic Scholar limiter.

## Reproduction on main

Recorded in the ticket before any product change (job `1792234191372-19559`,
the macOS gate host, at `4ce343291` = origin/main `40483dcab` + tests/fixture
only):

- The gap command `biomcp --json batch article 22663011 --mode detail
  --sections fulltext --pdf` exits 2 before any provider request:
  `unexpected argument '--pdf' found`.
- All four recorded tests failed: the two batch tests (exit 2 at clap), the
  MCP typed-get wire test (`unknown article get field: pdf`), and the typed
  mapper/schema unit test (no `pdf` property, no `--pdf` in argv).

## Change

- `src/cli/system/mod.rs` + `src/cli/system/batch.rs`: `batch` gains `--pdf`.
  Preflight refuses it for non-article batches, for compact article batches,
  and without the `fulltext` (or `all`) section, mirroring `get article`'s
  rule so the refusal happens before any provider request — the existing
  no-provider-work sentinel matrix now covers the three refusals.
- `src/cli/system/dispatch.rs`: article detail batches pass
  `ArticleGetOptions { allow_pdf: args.pdf, .. }` into the shared
  `entities::article::get`, so batch rides the same limiter, page, and size
  limits as `get article` with no second code path. Per-article reporting is
  the shared surface: `section_outcomes.fulltext` and the `_meta
  .section_sources` Full Text row name the source each article used (Europe
  PMC for an XML winner, Semantic Scholar for a PDF winner), and
  `full_text_coverage.attempts` carries 2142's part-naming reasons
  (`open_pdf_download_failed`, `open_pdf_text_unusable`, …) when the PDF step
  ran and failed. An XML winner records no PDF attempt: the ladder stopped at
  its first full-text rung.
- `src/mcp/shell.rs` + `src/mcp/shell/typed_get.rs`: the typed article get
  gains the boolean `pdf` field (schema default false), mapped to the `--pdf`
  flag after the sections. The fulltext-section rule stays with the shared
  detail path, so CLI and MCP report it identically; other entities keep
  rejecting the field before any provider work.
- Fixture `spec/fixtures/setup-article-fulltext-source-fixture.sh`: the
  publisher listener and the generic Semantic Scholar paper route log
  timestamped lines (`publisher-pdf:<path>@<ts>`, `s2:paper:<pmid>@<ts>`).
  No recorded reply or testdata file changed, so no receipts boundary pin
  moved.
- `crates/biomcp-mcp-contract-client`: `ArticleFulltextFixture` reads the
  fixture's `publisher-url` file and `article_fulltext_fixture_env` exports
  `BIOMCP_S2_PDF_FIXTURE_ORIGIN`, so contract tests can reach the vouched
  publisher PDFs the way the spec fixture env already did.
- Tests: `tests/article_batch_outcome.rs` gains
  `batch_article_detail_pdf_fetches_three_papers_under_one_limiter`
  (three PDF-path articles through one process: text from the vouched PDF, a
  download failure named `open_pdf_download_failed`, an extraction failure
  named `open_pdf_text_unusable`; the request log shows all three Semantic
  Scholar paper lookups came from the one run, and consecutive
  publisher-origin PDF downloads keep the limiter's per-origin interval with
  a 10 ms schedule allowance) and
  `batch_article_detail_pdf_names_the_fulltext_source_each_article_used`
  (an XML winner keeps `Europe PMC`, a PDF winner names
  `Semantic Scholar PDF` in manifest, section outcome, and provenance row).
  `tests/rmcp_client_contract.rs` gains
  `typed_article_get_accepts_pdf_fulltext_over_stdio` (typed `get` with
  `pdf: true` over stdio: PDF data for the winner, the named failure for the
  download-failure paper). `src/mcp/shell/typed_get_tests.rs` pins the
  mapping, the schema property, and the article-only rejection;
  `src/mcp/shell.rs` pins `pdf` in the typed-get root properties;
  `src/cli/system/batch.rs` pins the preflight matrix and the clap parse;
  `src/cli/system/tests.rs` pins the help line and example;
  `src/cli/list/tests/pages.rs` keeps its pinned sentence intact with the
  batch line added beside it.
- Docs: `docs/user-guide/article.md` (the batch and MCP shapes beside the
  existing `--pdf` rule), `docs/user-guide/cli-reference.md` (example,
  modifier note, batch prose), `src/cli/commands.rs` (batch NOTES + example),
  `src/cli/list/literature.rs` (article skill page), and
  `spec/entity/article.md` gains the reader-facing section
  "Detail Batch PDF Fallback Runs Under One Limiter" plus the five `--pdf`
  refusal lines in the preflight block. `CHANGELOG.md` carries the New
  features bullet.

## Spacing observability (why the test reads the publisher origin)

The fixture signal `BIOMCP_TEST_UNPACED_ORIGIN` is the only thing that
validates a plain-HTTP loopback Semantic Scholar base, and it also disables
pacing for that exact origin (`RateLimiter::wait_for_url` returns before key
resolution). Recorded replies therefore cannot observe the 2 s (1 s keyed)
`policy:semantic-scholar` interval on paper-detail requests: the coupling is
airtight by design. What one batch run can observe: all paper lookups come
from the one process (the `s2:paper:` log), and the provider-vouched PDF
downloads hit the second loopback origin, which the signal does not match —
those pace under the limiter's per-origin default key (100 ms), per 2142's
finding that downloads pace at the publisher origin's interval, not the
Semantic Scholar prefix interval. The batch test asserts that real spacing;
the Semantic Scholar policy interval for discovery stays pinned by
`article_source_urls_keep_their_live_pacing_policies` and
`semantic_scholar_interval_uses_key_aware_values`.

## Green evidence

Code tip `6a79b2796` on `ticket/2151-fetch-pdf-full-text-in-batch-under-one-limiter`
(base: origin/main `40483dcab`; the branch's earlier tips `4ce343291`,
`e2a4d9816`, `9331740ed`, `e6e9c3e99`, `16d286b2a`, `82200d62c`, `ffc561973`,
`e565288f5` carry the red record, the fixes, and two help-pin corrections):

| Job | Machine | Command | Verdict |
| --- | --- | --- | --- |
| 1792234191372-19559 | macOS gate host | gap command + targeted nextest at `4ce343291` (main + tests) | gap exits 2; 4/4 tests red |
| 1792234552282-43959 | macOS gate host | targeted nextest (the four acceptance tests) at `e6e9c3e99` | passed, 4/4 |
| 1792234474296-37193 | macOS gate host | targeted nextest (8-test set) at `9331740ed` | 7/8; the one failure was my wrong XML-winner expectation, fixed at `e6e9c3e99` |
| 1792236505658-31972 | macOS gate host | help + article-page pins at `e565288f5` | passed, 2/2 |
| 1792234711919-51673 | macOS gate host | `make lint` at `16d286b2a` | passed |
| 1792238179678-88363 | macOS gate host | `make lint` at `6a79b2796` | passed |
| 1792234794778-63383 | macOS gate host | `make spec` at `16d286b2a` | 59 passed, 2 failed — both the pre-existing host-specific clingen-erepo-frozen page (see 2147's record); the new spec block passed |
| 1792238278992-14804 | macOS gate host | `make spec` at `6a79b2796` | same 59/2 shape, same pre-existing failures only |
| 1792237199451-3815499 | Linux gate host | `make test` at `6a79b2796` | passed (exit 0; full rust lane + python contract lane + strict docs build) |
| CI run 39029337278 | GitHub Linux | full ci.yml at `6a79b2796` | green — every executed lane (rust-tests, python-lane, spec-lane, lint, build, contracts); windows/release-panic/full-features/stress lanes skipped by workflow conditions |

## Findings

- Main moved twice during the build. The 2150 landing's blanket id move was
  repaired by the supervisor at `40483dcab` (2142's PDF test ids restored),
  and the two remaining full-features clippy lints were repaired through
  `8c1eae09a`/2153. At the time of this record main's tip `67c42424e` is red
  again: CI run 39028801637 fails rust-tests on
  `entities::article::variant_search::tests::no_entity_annotation_run_names_the_miss_instead_of_complete_empty`
  (variant_search.rs:6384, "a no-entity miss is not an outage"), which the
  clippy repair's twin-branch merge appears to have broken. Not this
  ticket's diff; the landing rebase needs a green main first.
- The macOS gate host also fails
  `bootstrap_parent_fsync_failures_fail_closed_before_any_store_is_returned`
  (gencc) at plain main `40483dcab` run in isolation (`Store::open()` →
  `Err(Invalid)` at gencc/tests.rs:901); CI Linux passes it. Together with
  the clingen-erepo spec page, the macOS gate host has two pre-existing
  defects to route to its owner.
- The `--pdf` refusal examples in batch preflight deliberately point at
  `batch article ... --sections fulltext --pdf`; the shared detail path's
  own message still points at `get article`, which remains correct there.
- CHANGELOG carries the bullet at the top of the 0.9.3 New features list —
  the usual landing-rebase coupling; everything else in this record is
  coupling-safe.
