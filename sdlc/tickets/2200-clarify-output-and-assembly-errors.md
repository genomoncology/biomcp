---
priority: 3
---

# 2200 — Clarify output and assembly errors

Status: OPEN.

Milestone: 0.9.4

## Outcome

Error messages say what went wrong.

- Evidence: release review of 7025e117: `--output ./` gives `Device or resource busy (os error 16)` and an existing directory gives `Not a directory` instead of saying the output is a directory. A not-found variant with `--assembly hg38` embeds `(attempted GRCh38; upstream HTTP 404)` inside the quoted ID and suggests `biomcp search variant` with no arguments. The FDA orphan health row gives no reason now that the site no longer redirects. The GWAS health probe gets HTTP 410 while search works (also in v0.9.2).
- Proof: outside-in cases for each message.
