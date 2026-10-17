---
priority: 3
---

# 2183 — Check why the PDF step finds no text for 10615958

Status: OPEN.

Milestone: 0.9.3

## Outcome

We know whether the Semantic Scholar PDF for PubMed 10615958 has a text layer, and BioMCP extracts it if it does.

- Evidence: research's spot check: `get article 10615958 fulltext --pdf` reports `open_pdf_text_unusable`. The PDF step works elsewhere: on 2026-10-17 `get article 22663011 fulltext --pdf` returned 40,114 bytes of text. 10615958 is a 1999 paper and may be an image-only scan.
- Changes: none unless the PDF has text. If it does, claim the PDF text extraction files and `CHANGELOG.md`.
- Proof: record the PDF's text-layer check. If it has no text layer, record that and close; the named reason is correct. If it has text, a recorded-reply test extracts it, and fails on current main.

Coordinator outcome before code: Inspect the actual Semantic Scholar PDF text layer and extraction result for the reported paper. If image-only, record the correct existing refusal and close under the approved conditional outcome. Claim code and a failing recorded-reply extraction test only if a usable text layer exists. Before product edits, reproduce on current main through the build job and report the exact command/output, proposed implementation/test files and the outside-in failing proof. Root records that evidence and scope before implementation. Developers write no ticket or status records.

## Progress

- 2026-10-17 started


## Coordinator reproduction and scope

Qualified CLI source `926b5786d400d3ec8b54c929701c1e46c62b5a95` differs from current main `90c00c371` only by ticket records. CLI SHA256: `e9992fadcb93e290a6c310827a14368444dcd1305784661c659c46311ed9083d`.

Job 2183/1792273540992-76331 ran `biomcp --json --no-cache get article 10615958 fulltext --pdf`, exit0, reproducing unavailable/open_pdf_text_unusable. Metadata identified public PDF and Semantic Scholar paper26ecb6bda482ce56fcc4e4ec28da43121a15a05e. Credential-free publisher request returned403. Job 2183/1792273647491-84897 confirmed403 using the application's normal user agent: text/html,5632bytes,SHA256 7116f59e56752bcaa957ad093db208e6c26a682d51d2bb3bda6b5e8b0cadd19d, HTML signature present, PDF absent. Separate public API lookup returned429; no bypass/repeated rate-limit requests.

Actual PDF bytes never arrived. PDF checksum,size,pages,text-layer status unknown. No image-only conclusion or extraction-bug claim is justified. PM decision needed: obtain research's retained authorized PDF bytes or decide disposition. Evidence collection stopped; no code/test/changelog edits. Ignored build-job .review-artifacts/2183 retains captured responses pending disposition.
