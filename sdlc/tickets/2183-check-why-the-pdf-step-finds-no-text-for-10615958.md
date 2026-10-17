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
