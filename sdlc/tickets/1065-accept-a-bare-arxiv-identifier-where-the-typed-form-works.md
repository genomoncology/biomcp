---
flow: build
priority: 8
---

# Accept a bare arXiv identifier where the typed form works

Filed from `sdlc/issues/2026-08-27-article-authors-rejects-a-bare-arxiv-identifier.md`.

`article authors 2110.01406` errors while `article authors arXiv:2110.01406`
works. Every other identifier family in the article helpers is accepted
bare (PMID, PMCID, DOI), and a bare arXiv number — digits, a dot, digits —
is unambiguous against every other supported form. The error message
teaches the fix, so this is a papercut, but papercuts on a brand-new pivot
surface are what users hit on day one.

## Done when

- `article authors 2110.01406` behaves exactly like `article authors
  arXiv:2110.01406` — same card, same authors — wherever the S2 article
  helpers run.
- The acceptance rule is pinned by a test covering the bare form, and the
  existing typed form is unchanged.
- No other identifier family's parsing changes.

Filed as build, not quickfix: green suite; the acceptance proof is authored.
