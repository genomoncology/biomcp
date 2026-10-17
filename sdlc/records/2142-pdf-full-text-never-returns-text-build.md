# 2142 build record — return text from the PDF full-text step

## Report being fixed

`biomcp --json --no-cache get article 10615958 fulltext --pdf` on 0.9.2 reported
`source_unavailable` at the PDF step and returned no text; the probe saw all 12
papers that reached the PDF step get no text (7 `source_unavailable`, 5
`no_content`).

## Reproduction on main (before any code)

Job `1792206207332-78841` (the macOS gate host) at `b138f6097` = origin/main ran
the ticket's exact command: exit 0, PDF attempt
`{"source_kind": "pdf", "coverage": "unavailable", "outcome": "unavailable",
"reason": "source_unavailable"}`, `full_text_path: null`. Recorded in the
ticket before any product change.

## Root cause — the open-PDF link, refused before any download

Semantic Scholar's live reply for PMID 10615958 (recorded 2026-10-16, read-only
public GET) lists
`openAccessPdf.url = https://diabetesjournals.org/diabetes/article-pdf/49/1/114/364646/10615958.pdf`.
The PDF step's outbound policy allowlisted only `pdfs.semanticscholar.org` and
`www.semanticscholar.org`, so every publisher-hosted open PDF was refused at
link validation: no request was sent, and the refusal was recorded as
`source_unavailable`. Not the download and not the text extraction — the link.

## Change

- `src/sources/provider_url_policy.rs`: a new `OriginRule::ProviderVouchedUrl`
  mode. `semantic_scholar_pdf()` now treats the listed link as provider-vouched:
  any HTTPS origin with a DNS-name host and the default port may answer, while
  every transport guard stays — credentials in the URL are refused, IP-literal
  hosts are refused, DNS answers that resolve to loopback/private/link-local/
  CGNAT/metadata addresses are refused in the resolver, and every redirect is
  revalidated by the same rules. `unsafe_test_origin` became
  `unsafe_test_origins` (a set) so a second exact-loopback fixture origin can
  stand in for the publisher beside the shared unpaced origin.
  `BIOMCP_S2_PDF_FIXTURE_ORIGIN` selects that origin; it must be an exact IP
  loopback, pathless, or the policy errors (the ClinGen LDH/CSpec precedent).
- `src/entities/article/mod.rs` + `src/entities/article/fulltext.rs`: the PDF
  step's failure reasons now name the failed part — `open_pdf_link_unavailable`
  (discovery failed), `open_pdf_link_absent` (no open PDF listed),
  `open_pdf_link_rejected` (refused before contact; new
  `FulltextStepOutcome::Refused` so a policy refusal cannot masquerade as a
  download failure), `open_pdf_download_failed`, `open_pdf_download_absent`
  (404/204), `open_pdf_text_unusable`. XML/HTML attempts keep their vocabulary.
- Fixture `spec/fixtures/setup-article-fulltext-source-fixture.sh`: a second
  listener (the publisher origin) serves the captured open PDF and the download
  and extraction failure replies; new PMIDs 22663026 (link → text),
  22663027 (link refused: credentialled URL, never echoed),
  22663028 (download 500), 22663029 (bytes that are not a parsable PDF);
  the fixture env exports `BIOMCP_S2_PDF_FIXTURE_ORIGIN`.
- `testdata/sources/plos/pone-0342026-printable.pdf`: the captured open-access
  PLOS ONE correction PDF (PMID 41615898, DOI 10.1371/journal.pone.0342026,
  CC BY), byte-faithful with a receipt and byte-identical to the PMC OA
  package copy; the receipts audit passes with no fixture-count pin change
  (`code_keys_checked` stays 109, audited files 378 → 379) and the receipts
  digest repin lives in `tools/zero-coupling-historical.json`.
- Tests: `tests/article_asset_terminal.rs` —
  `publisher_open_pdf_link_downloads_and_returns_its_text` (recorded reply:
  downloads the captured PDF through the publisher origin, saves its text,
  manifest `pdf`/`Semantic Scholar PDF`/`pdf_fallback_used`),
  `open_pdf_failures_name_the_part_that_failed` (all six reasons against
  recorded shapes, and the refused link's credentials never echo). Unit tests:
  the vouched policy accepts publisher hosts and keeps the transport guards;
  unsafe URLs are `Refused` before contact without echoing.
- Docs: `docs/user-guide/article.md` states the publisher-origin rule and the
  six part-naming reasons. `CHANGELOG.md` carries the Fixes bullet.

## Red on main

Job `1792207718537-2221558` (the Linux gate host) ran the two new terminal tests at
`a2c6f5750` = origin/main + fixture/tests/docs only, no product change (`0 passed, 2
failed`; the job's own exit line read 0 only because the shell pipeline
swallowed nextest's status, so later gates run with `pipefail`):
`publisher_open_pdf_link_downloads_and_returns_its_text` FAILED (on main the
publisher-origin link is refused, `reason: "source_unavailable"`, no manifest,
no saved text) and `open_pdf_failures_name_the_part_that_failed` FAILED (none
of the six part-naming reasons exist on main).

## Green evidence

Final tip `d34868ba4` ("Capture a readable open PDF and move the outage case to
a no-winner paper"; the two earlier branch tips `69c4263cb` and `3cfd2199c`
carried a first capture whose text layer the production extractor cannot read
and an outage case whose paper has a PMC HTML winner, so the PDF step never
ran — both caught by CI's rust lane and fixed in the final tip):

| Job | Machine | Command | Verdict |
| --- | --- | --- | --- |
| 1792213118704-3620 | the macOS gate host | scratch diagnostic: candidate PDF through the real CLI | clean text, capture swapped |
| 1792213389724-11900 | the macOS gate host | `make lint` at `d34868ba4` | passed |
| 1792214725519-38339 | the macOS gate host | `make lint` at `c878b691b` | passed |
| 1792213392668-12436 | the macOS gate host | `make spec` at `d34868ba4` | 59 passed; the only 2 failures are the pre-existing clingen-erepo page (below) |
| 1792212451039-54350 | the macOS gate host | `make spec` at plain main `b138f6097` | same 2 clingen-erepo failures — pre-existing, not this ticket |
| «yellow-targeted» | the Linux gate host | targeted nextest: the new tests at `d34868ba4` | CI's rust lane covers the same tests green at both tips; the queued yellow runs are noted below |
| «yellow-test» | the Linux gate host | `make test` at `c878b691b` | «fill» |
| CI run 39004824318 | GitHub Linux | full ci.yml at `d34868ba4` | python-lane failed (3 contract gaps below), all other executed lanes green |
| CI run 39006201972 | GitHub Linux | full ci.yml at `c878b691b` (final tip) | green — every executed lane passed |

### Live probe (read-only public GETs, the macOS gate host, job 1792213938177-72006, `d34868ba4`)

- The ticket's exact command `biomcp --json --no-cache get article 10615958
  fulltext --pdf` exits 0 and the PDF attempt now records
  `{"reason": "open_pdf_text_unusable", "coverage": "unusable"}`: the
  diabetesjournals.org link is downloaded instead of refused, and the
  publisher's gated bytes are named as an extraction failure instead of a
  false `source_unavailable` (Semantic Scholar lists that PDF with status
  CLOSED).
- `get article 41615898 fulltext --pdf` (the captured correction paper) returns
  full text through the earlier JATS rung with its text saved — the ladder
  order is unchanged.

## Seams for 2151 (batch PDF full text)

- Entry point: `src/entities/article/fulltext.rs` — `pdf_discovery_attempt()`
  (link discovery from the S2 enrichment), `try_resolve_pdf()` (validate →
  `provider_url_client("semantic-scholar", &policy)` → download → extract),
  `resolve_fulltext()`'s PDF arm (attempt recording). Batch should drive the
  same functions so per-article failures keep the six part-naming reasons.
- Limits in force: `PDF_MAX_BODY_BYTES` (20 MiB), `PDF_PAGE_LIMIT` (12 pages),
  `crate::sources::with_response_body_limit`, `apply_no_store` (the PDF GET is
  never served from the HTTP cache).
- Limiter: the shared `RateLimitMiddleware` in `src/sources/mod.rs`; the PDF
  download rides the provider-URL client, and a publisher URL paces at the
  per-origin default interval, not the Semantic Scholar prefix interval. A batch
  under "the one in-process Semantic Scholar limiter" (2151) must account for
  that: discovery hits `api.semanticscholar.org` (S2 policy interval), downloads
  hit each publisher origin (default interval).
- Policy seam: `ProviderUrlPolicy::semantic_scholar_pdf()` (vouched origins) and
  `BIOMCP_S2_PDF_FIXTURE_ORIGIN` for recorded replies; the ratchet test
  `provider_url_consumer_ownership_ratchet_names_every_fetch_site` pins the
  fetch site to `semantic_scholar_pdf()`.

## Findings

- The python lane caught three contract gaps at `d34868ba4`, all fixed at
  `c878b691b`: the new env var needed a fixture-only classification, the replay
  env needed it listed as a non-provider override (it is an allowlist entry the
  article fixture exports, like the health probe hook), and a blank line had
  split the changelog's Fixes bullet list.
- `make spec` fails its clingen-erepo page on the macOS gate host at plain main
  (`b138f6097`, job `1792212451039-54350`) exactly as at this ticket's tip: a
  pre-existing gate-host breakage, not this change (CI's Linux spec lane passes
  the page). Whoever owns the macOS gate host should look at
  `run-clingen-erepo-fixture.sh` there.
- The first capture (PLOS Biology pbio.3000076, byte-identical in the PMC OA
  package) carries a text layer the production extractor (unpdf) cannot map —
  its words arrive as glyph-index fragments. The landed capture (PLOS ONE
  pone.0342026, also byte-identical to its PMC OA package copy) extracts
  cleanly. Publisher PDF quality varies; `open_pdf_text_unusable` is the
  honest answer for unreadable files.
- The 0.9.2 probe's 5 `no_content` cases are honest absences that were badly
  named: with this change a paper with no listed open PDF reports
  `open_pdf_link_absent` and a 404/204 download reports
  `open_pdf_download_absent`.
- The live 10615958 PDF carries `status: CLOSED` in Semantic Scholar's payload;
  its publisher host may still refuse the download. Whatever it answers, the
  step now names the part that failed.
