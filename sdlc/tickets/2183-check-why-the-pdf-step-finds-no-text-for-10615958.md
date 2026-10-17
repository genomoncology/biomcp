---
priority: 3
---

# 2183 — Check why the PDF step finds no text for 10615958

Status: COMPLETE.

Milestone: 0.9.3

Reviews: revision 89a1dcc, accept

Landed: 89a1dcc (fast-forward: Accepted work reached main in reviewed combined research-fix landing bf9916b; completion records this source batch without another source change.)

## Outcome

PDF full-text attempts distinguish an unavailable response from actual PDF bytes with no usable text. A refused request or HTML response reports access refused or not a PDF; `open_pdf_text_unusable` is reserved for actual PDF bytes that yield no usable text.

- Evidence: the public publisher request for 10615958 returned HTTP403 HTML; the CLI reported open_pdf_text_unusable. The real PDF's text layer is unknown and need not be obtained.
- Keeps: successful PDF text extraction, bounded downloads, existing retry/access controls, no OCR or access bypass.
- Changes: claims `src/entities/article/fulltext.rs` `tests/article_asset_terminal.rs` `spec/fixtures/setup-article-fulltext-source-fixture.sh` `tests/fixtures/article/fulltext/pdf/blank.pdf` `CHANGELOG.md`. Use distinct PDF routes and test functions; no publisher download or transfer between teams.
- Proof: a local recorded server returning403 HTML produces an unavailable/access-refused result; an HTML non-PDF response is not text_unusable; an actual PDF with no text layer remains text_unusable. Reproduce the current CLI failure before source edits.

PM decision: 2026-10-17. Stop chasing publisher bytes. Correct the reported reason using local fixtures for403 HTML and a real PDF without text, preserving text_unusable only for actual PDF bytes.

## Coordinator reproduction and scope

Qualified CLI source `926b5786d400d3ec8b54c929701c1e46c62b5a95` differs from current main `90c00c371` only by ticket records. CLI SHA256: `e9992fadcb93e290a6c310827a14368444dcd1305784661c659c46311ed9083d`.

Job 2183/1792273540992-76331 ran `biomcp --json --no-cache get article 10615958 fulltext --pdf`, exit0, reproducing unavailable/open_pdf_text_unusable. Metadata identified public PDF and Semantic Scholar paper26ecb6bda482ce56fcc4e4ec28da43121a15a05e. Credential-free publisher request returned403. Job 2183/1792273647491-84897 confirmed403 using the application's normal user agent: text/html,5632bytes,SHA256 7116f59e56752bcaa957ad093db208e6c26a682d51d2bb3bda6b5e8b0cadd19d, HTML signature present, PDF absent. Separate public API lookup returned429; no bypass/repeated rate-limit requests.

Actual PDF bytes never arrived. PDF checksum,size,pages,text-layer status unknown. No image-only conclusion or extraction-bug claim is justified. PM decision needed: obtain research's retained authorized PDF bytes or decide disposition. Evidence collection stopped; no code/test/changelog edits. Ignored build-job .review-artifacts/2183 retains captured responses pending disposition.

Coordinator next action: the earlier evidence-only blocker is superseded by the PM classification ruling above. Start a fresh developer on the local-fixture reason fix; leave the prior inspection facts intact as evidence.

Coordinator scope approval: reuse existing open_pdf_download_failed for unavailable non-PDF responses; actual signature-bearing PDF without usable text remains open_pdf_text_unusable. Existing HTTP403 rejection is a passing control; capture a real failing HTML response boundary before source edits. The earlier direct403 request does not establish the HTTP status received by the original CLI PDF request.

Actual outside-in RED:2183/1792274086821-63100 used the qualified current-main CLI and recorded local responses. HTTP403 HTML already reports download_failed/unavailable. HTTP200 HTML, both text/html and misleading application/pdf, incorrectly reports text_unusable/unusable. Actual blank PDF remains text_unusable; existing good publisher PDF produces full text. Job exited1 on the HTML classification assertions before source edits; fixture/test patch SHA2566db0634a57d17b68e0d3a00404a279705ce2e8469f87947b5d5ed20f69116563. Authorize minimal non-signature response classification change with the actual blank PDF fixture in scope.

GREEN2183/1792274113028-65337 passed formatting, all3 selected outside-in PDF integration cases and make lint including Clippy and quality checks. Tested product patch SHA256d46c0b5abdc9b4c1353bdde7549f5de7600fe70258c82859476375c4866faad9 matches local sourcee1ca7428dc0567105600b10de04b6d42e3b37efd. HTTP200 HTML now reports existing download_failed/unavailable regardless of MIME; HTTP403 remains correct; actual blank PDF remains text_unusable and successful PDF extraction remains covered. The obsolete MIME helper and its redundant detector test were removed in favor of the actual CLI regression. No publisher bytes were fetched. Fresh review follows.
