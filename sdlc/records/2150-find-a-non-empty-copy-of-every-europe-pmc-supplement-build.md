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
  - `article_asset_bytes_from_resolution` answers a bare file name that
    several routes refused with the strongest single refusal (the same
    priority order the fold uses) instead of a bare not-found, so the new
    article-page rows cannot hide 2146's empty-response refusal; coverage
    folds per (canonical identity, source document) so one route's outcome
    never hides another's reason while delivered copies still merge.
- `src/sources/pmc_oa.rs`: `object_bytes` is crate-visible for the fallback;
  the media-object filename extraction moved to `media_object_filename`,
  which `archive_package` now shares.
- Fixtures: `22663031` mirrors the Word case (empty Europe PMC member,
  package broken by a 9 MiB sibling, bucket object with the real docx),
  `22663032` the wrong-type case (vendor HTML under a .docx name in both
  archive routes, article page gated), `22663033` the PDF case (zero-byte
  PDF member beside a figure with bytes; package holds the real PDF).
  Existing xlsx/docx fixture bytes gained committed leading bytes (`PK`)
  because the binary they stand in for now answers a type check. The three
  fixture articles first shipped as 22663026–28 and moved to 22663031–33
  when 2142's landed PDF-failure fixtures took 22663026–30; the earlier gate
  jobs in this record ran under the old ids with byte-identical fixtures.
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
| 1792207717808-50107 | macOS gate host | live reproduction at `43252305f` (main) | both defects shown; PDF control good |
| 1792207792924-54817 | macOS gate host | live downloads at `43252305f` (main) | wrong-type exits 0 with the HTML bytes; PDF control good |
| 1792208463514-2224639 | Linux gate host | targeted nextest at `ed7a71343` (main + tests/fixture) | failed: Word and wrong-type red, PDF control green |
| 1792221301105-3455609 | Linux gate host | targeted nextest (8 tests: the three new terminal tests, 2146's two terminal tests, three unit tests) at `326ab7fd1` | passed, 8/8 |
| 1792224468485-3601840 | Linux gate host | `make test` at `326ab7fd1` | passed |
| 1792225823259-3666708 | Linux gate host | `make lint` at `6a1b8ac6a` | passed |
| 1792226733989-3671083 | Linux gate host | `make spec` at `6a1b8ac6a` | passed |
| 1792226898757-77664 | macOS gate host | live probe with the fixed binary at `6a1b8ac6a` | see below |
| CI run 39017486462 | GitHub Linux | full ci.yml at the pre-rebase tip `6a1b8ac6a` | green (every executed lane) |
| 1792227501526-3690711 | Linux gate host | targeted nextest (the three new terminal tests and 2146's two) at the first post-rebase tip `ef512ee27` | passed, 5/5 |
| 1792229610310-3740468 | Linux gate host | targeted nextest (the healed no-entity test and the three new terminal tests) at the final tip `c58e0d059` | passed, 4/4 |
| CI run 39021581292 | GitHub Linux | full ci.yml at the final tip `c58e0d059` | green (every executed lane) |
| CI run 39019163347 | GitHub Linux | full ci.yml at `71674ff52` (record-only over `6a1b8ac6a`) | green |

Superseded failures on the way (all understood, none open): CI 39005740828
and 39010671915 were compile errors fixed before the next push; CI 39006024620
and 39009800755 caught the two real defects below (both fixed in
`1f65999b6`/`326ab7fd1` and re-proven by the 8/8 run); CI 39012916822 was a
spec-lint short-match literal; yellow 1792214563558-2790300 and
1792219137243-3407725 ran stale tips and failed the same fixed defects.

### Live probe at the tip (read-only public GETs, job 1792226898757-77664)

- Word case (30224644): the assets manifest now lists eleven retrievable
  copies through the bucket fallback (main listed none), and
  `asset NIHMS1501645-supplement-3.xlsx --output /tmp/sup3.xlsx` downloads
  the real 397,947-byte workbook (PK header). The reported 46 MiB
  `NIHMS1501645-supplement-2.docx` itself refuses with exit 1 and no file;
  its coverage rows read Europe PMC `empty_response`, PMC OA
  `source_unavailable` (the 8 MiB per-object budget), PMC
  `pmc_proof_of_work` (the page route's gate).
- Wrong-type case (33332384): `asset pgen.1009231.s004.docx` now exits 1 and
  writes no file (main exited 0 and wrote 40,919 bytes of HTML).
- PDF control (27149063): unchanged — exit 0, the good 18,070-byte PMC OA
  copy, SHA-256 `3e3ab5a0…9ef0`.

## Findings

- The Word case's 46 MiB supplement still cannot download whole: the 8 MiB
  per-object budget refuses it at the bucket as it refused the package, so
  the manifest says no usable copy exists with three honest route reasons.
  The ticket's acceptance explicitly allows "the manifest says none exists";
  a follow-up could raise the per-file budget for the fallback if the
  research team needs that one file tonight.
- A bare file name that several routes refused answers with the strongest
  single reason (proof-of-work above wrong-type above empty above
  unavailable). Live, that means the wrong-type article's bare name names
  the article-page bot gate while the route keys answer `unusable_asset`;
  the recorded test pins the `unusable_asset` refusal through the stable
  route key, the same key form 2146 established.
- Coverage now folds per (canonical identity, source document): the bucket
  route and the linked page route share an identity so a delivered copy
  still merges into one asset, but their refusals stay separate rows so one
  route's outcome cannot hide another's reason. This was forced by the
  fallback: without it the page's proof-of-work row swallowed the PMC OA
  `unusable_asset` row on the wrong-type article.
- Existing xlsx/docx fixture bytes gained committed leading bytes (`PK`)
  because the binaries they stand in for now answer a type check; substring
  assertions elsewhere still match, and 2146's tests are green at the tip.
- `make lint` still cannot run on the macOS gate host (no Darwin-arm64
  archive for the pinned shellcheck/actionlint; same finding as 2146), so
  the lint gate ran on the Linux gate host and in CI.
- The pre-existing `unused import: std::fs` warning for
  `src/cache/manager.rs` still appears in nextest builds (present on main
  since `d59599c88`); clippy's lane passes.
- Rebased twice as main moved mid-build: first onto `159c68610` (2131)
  with no conflicts, then onto the healed `8988f5a31` (2140's test pins)
  with no conflicts. The branch tip is `c58e0d059`.
- **Main's rust-tests lane went red mid-build from 2131's new test**
  `entities::article::variant_search::tests::no_entity_annotation_run_names_the_miss_instead_of_complete_empty`
  ("a no-entity miss is not an outage", `variant_search.rs:5433`): it failed
  on pure main in CI (run 39015559033 at `159c68610`, lanes rust-tests and
  full-features) and on the Linux gate host three runs of three (job
  1792228315381-3737081 at `159c68610`), while one green CI run at the same
  SHA showed it flaked by environment. My branch's CI at the first
  post-rebase tip `ef512ee27` (run 39019340388 plus a `--failed` rerun) was
  red only on that test; my diff does not touch variant search. On the
  coordinator's hold, landing waited for 2140's cure; after 2140 landed at
  `8988f5a31` the branch rebased onto the healed main as `c58e0d059`, where
  the test passes again (job 1792229610310-3740468) and the full ci.yml run
  39021581292 is green.
