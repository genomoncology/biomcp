---
priority: 2
---

# 2252 — Make variant articles Markdown match its JSON

Status: OPEN.

Milestone: 0.9.4

Reviews: revision eb5b7e2, accept

Reviews: revision 4d6e043, accept

## Outcome

Changes: `src/cli/variant/articles.rs`, `src/render/markdown/article.rs`, `src/render/markdown/mod.rs`, `templates/article_search.md.j2`, `tests/test_variant_article_source_status.py`, `CHANGELOG.md`, `sdlc/tickets/2251-report-publisher-elided-references-as-empty.md`, `sdlc/tickets/2252-make-variant-articles-markdown-match-its-json.md`, `sdlc/tickets/2253-document-full-text-access-groups-and-reason-codes.md`, `sdlc/tickets/2254-fix-article-help-lists-and-condition-names.md`.

The Markdown source plan and source status of `biomcp variant articles` report the same sources and states as its JSON.

## Evidence

Reproduced on 0.9.3: `biomcp variant articles "BRAF V600E" --limit 5` prints "Source plan: candidates [PubTator3, Europe PMC, Semantic Scholar]; enrichment [Semantic Scholar]" and "Semantic Scholar: disabled" with `S2_API_KEY` set, while JSON `source_status` shows LitVar2, PubMed and Semantic Scholar routes ok and rows list PubMed and Semantic Scholar.

## Keeps

Render Markdown from the same source-status data as JSON; no second computation.

## Proof

A replay test renders both formats for one run and checks they name the same sources and states, with and without the Semantic Scholar key.

## Progress

- 2026-10-19 started

## Verification

The baseline replay reproduced missing Markdown source states. Final job 2252/1792428114066-33654 passed six JSON/Markdown parity cases and Python lint. Job 1792427992444-97476 passed the binary build, existing renderer cases, formatting and all-target lint, then stopped on a malformed audit invocation; corrected audit job 1792428249914-82412 passed. Temporary successful Semantic Scholar replies are explicitly authored, committed captures remain unchanged, and replay misses fail. No live lookup, full gate or claim of integration with the pending gene-alias change.

Keep: experiments/795-biomcp-2252-source-status — retain checked patches, raw baseline and slow/failed attempts, corrected checks and audit output.

Coordinator repair grant: shared gate 2247/1792431202813-16176 failed four source-status cases because the combined gene-alias implementation adds a MyGene lookup absent from the authored replay. Repair only tests/test_variant_article_source_status.py to explicitly author the optional gene lookup or disable it consistently with this source-status test scope; preserve source-state, coverage, request and replay-miss assertions. Do not change product behavior. Keep the complete failed gate and focused combined proof in experiment 795.
