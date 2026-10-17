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
- `testdata/sources/plos/pbio-3000076-printable.pdf`: the captured open-access
  PLOS Biology PDF (PMID 30571682, DOI 10.1371/journal.pbio.3000076, CC BY),
  byte-faithful with a receipt; the receipts audit passes with no boundary pin
  change (`code_keys_checked` stays 109, audited files 378 → 379).
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

Job `«red-job»` (the Linux gate host) ran the two new terminal tests at
`a2c6f5750` = origin/main + fixture/tests/docs only, no product change:
`publisher_open_pdf_link_downloads_and_returns_its_text` FAILED (on main the
publisher-origin link is refused, `reason: "source_unavailable"`, no manifest,
no saved text) and `open_pdf_failures_name_the_part_that_failed` FAILED (none
of the six part-naming reasons exist on main).

## Green evidence

| Job | Machine | Command | Verdict |
| --- | --- | --- | --- |
| «fill» | «host» | targeted nextest: the new tests | «fill» |
| «fill» | «host» | `make test` at «tip» | «fill» |
| «fill» | «host» | `make spec` at «tip» | «fill» |
| «fill» | «host» | `make lint` at «tip» | «fill» |
| CI run «fill» | GitHub Linux | full ci.yml at «tip» | «fill» |

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

- The 0.9.2 probe's 5 `no_content` cases are honest absences that were badly
  named: with this change a paper with no listed open PDF reports
  `open_pdf_link_absent` and a 404/204 download reports
  `open_pdf_download_absent`.
- The live 10615958 PDF carries `status: CLOSED` in Semantic Scholar's payload;
  its publisher host may still refuse the download. Whatever it answers, the
  step now names the part that failed.
