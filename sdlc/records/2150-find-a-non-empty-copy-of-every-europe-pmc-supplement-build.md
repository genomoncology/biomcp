# 2150 build record — find a usable copy of every supplement

## Reports being fixed

Three research reports on 0.9.2 (ticket evidence, PMCIDs resolved live):
PMC4858276 (PDF), PMC6168352 (Word), PMC7787686 (wrong type).

## Reproduction on main — both defects are real

Live reproduction at `43252305f` = origin/main on the macOS gate host,
read-only public GETs through the built dev binary:

- Job `1792207717808-50107`: `biomcp --no-cache get article 30224644 asset
  NIHMS1501645-supplement-2.docx` exits 1 `article asset not found`; the
  assets manifest lists no retrievable copy. Coverage shows the docx as
  `empty_response` (Europe PMC) beside `source_unavailable` rows; PMC OA is
  degraded. The same job recorded the wrong-type manifest: 22663024-style
  shape aside, `pgen.1009231.s004.docx` lists as a PMC OA asset of 40,919
  bytes.
- Job `1792207792924-54817`: `biomcp --no-cache get article 33332384 asset
  pgen.1009231.s004.docx --output /tmp/wrong.docx` exits 0 and writes
  40,919 bytes starting `<!DOCTYPE html>`, SHA-256 `1ea833ea…87ee` — the
  exact reported bytes delivered under a .docx name. Control in the same
  job: the PDF case (27149063) already downloads the good 18,070-byte
  PMC OA copy, SHA-256 `3e3ab5a0…9ef0`.

Root cause for the Word case: the real supplement is 48,248,574 bytes. One
media object over the 8 MiB per-object budget makes `archive_package` fail
whole, so no package member is ever listed, and BioMCP never fetches the
bucket objects the retained manifest still names. The article-page route
answers with a bot-challenge page that the linked classifier records as
`source_unavailable` (the observed reCAPTCHA page carries no PoW marker).

## Change

- `src/entities/article/mod.rs`: new `ArticleAssetNamedOutcome::UnusableAsset`
  (JSON `unusable_asset`).
- `src/entities/article/assets.rs`:
  - `asset_bytes_match_filename` checks a download's leading bytes against
    the file name's extension (PDF within the first kilobyte, the zip family,
    the legacy Office container, gzip, and the common image types; text
    formats commit to nothing). A mismatch is refused exactly like an empty
    reply: exit nonzero from `asset` with the `unusable_asset` reason, no
    output file, never listed in `assets`, and the route keeps the asset-key
    form it would have carried. `PendingCoverage::zero_byte_download` became
    `refused_download` to cover both refusals; `named_outcome_priority` puts
    `UnusableAsset` beside `EmptyResponse`; the summary manifest builders
    filter wrong-type members the way they filtered empty ones; the asset
    manifest cache schema moves 2 → 3.
  - Every download route answers through the check: PMC OA package members,
    Europe PMC ZIP members, linked-file fetches, and Figshare downloads.
  - `supplement_fallback`: supplement file names an archive route listed
    without delivering a usable copy — empty members, wrong-type members,
    and every media object of a manifest whose package download failed — are
    looked up by name in the PMC open-data bucket (per-file `object_bytes`
    fetches under one 30 s budget and a 64 MiB aggregate, each object
    classified by the same empty/type rules, already-observed objects
    skipped) and, for names no declared JATS/page link covers, on the PMC
    article page's own file route (`/articles/instance/<id>/bin/<name>`
    through the linked machinery with its PoW/absent classification).
- `src/sources/pmc_oa.rs`: `object_bytes` is crate-visible for the fallback;
  the media-object filename extraction moved to `media_object_filename`,
  which `archive_package` now shares.
- Fixtures: `22663026` mirrors the Word case (empty Europe PMC member,
  package broken by a 9 MiB sibling, bucket object with the real docx),
  `22663027` the wrong-type case (vendor HTML under a .docx name in both
  archive routes, article page gated), `22663028` the PDF case (zero-byte
  PDF member beside a figure with bytes; package holds the real PDF).
  Existing xlsx/docx fixture bytes gained committed leading bytes (`PK`)
  because the binary they stand in for now answers a type check.
- Tests: three recorded-reply terminal tests (the Word case finds the bucket
  copy, the wrong-type case is refused with `unusable_asset` and writes no
  file, the PDF case prefers the PMC OA copy), unit tests for wrong-type key
  stability and refusal and for the committed-leading-bytes table.
- Docs: `docs/user-guide/supplementary-materials.md` gains the
  `unusable_asset` row, the refusal contract, and the fallback routes;
  `docs/user-guide/article.md` states the rule; `spec/entity/article.md`
  gains three reader-facing sections; `CHANGELOG.md` carries the Fixes
  bullet.

## Red on main

Recorded tests at `ed7a71343` = origin/main `43252305f` + fixture/tests
only, no product change (see Gates for the job):

- `empty_europe_pmc_supplement_finds_the_pmc_oa_bucket_copy` FAILED — on
  main no copy is listed; the supplement cannot be read at all.
- `wrong_type_supplement_reply_fails_the_download_and_writes_no_file`
  FAILED — on main the HTML bytes list as an asset and download with exit 0.
- `two_route_pdf_supplement_prefers_the_pmc_oa_copy` PASSED on main — the
  control case; 2146's preference already holds for a PDF, so this ticket
  regression-proofs it.

## Gates

| Job | Machine | Command | Verdict |
| --- | --- | --- | --- |
| (fill) | | | |

## Findings

- (fill)
