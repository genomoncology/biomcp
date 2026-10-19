---
priority: 2
---

# 2251 — Report publisher-elided references as empty

Status: OPEN.

Milestone: 0.9.4

## Outcome

Changes: `src/sources/semantic_scholar.rs`, `src/entities/article/graph.rs`, `src/entities/article/mod.rs`, `src/render/markdown/article.rs`, `tests/surface/test_article_graph_empty_contract.py`, `src/entities/article/graph/tests.rs`, `src/cli/article/tests/exact_lookup.rs`, `src/render/markdown/article/tests.rs`, `CHANGELOG.md`, `sdlc/tickets/2251-report-publisher-elided-references-as-empty.md`, `sdlc/tickets/2252-make-variant-articles-markdown-match-its-json.md`, `sdlc/tickets/2253-document-full-text-access-groups-and-reason-codes.md`, `sdlc/tickets/2254-fix-article-help-lists-and-condition-names.md`.

`biomcp article references` for a paper whose publisher elides its reference list returns an empty result that names Semantic Scholar and the elision, as the README promises, instead of a generic source failure.

## Evidence

Reproduced on 0.9.3: `biomcp article references 24200969 --limit 5` (also 22663011 and the help example 10.1056/NEJMoa1203421) prints "Error: API request to BioMCP source failed. Review source configuration and retry."; with `--json` the source reads "BioMCP source". Semantic Scholar's Graph API returns HTTP 200 with `data: null` and "The following paper fields have been elided by the publisher: {references}". README lines 326-328 promise an empty result.

## Keeps

Genuine Semantic Scholar failures still report as Semantic Scholar failures, never "BioMCP source". Apply the same handling to citations if the same reply shape occurs there.

## Proof

A replay test with the elided reply returns an empty result with the Semantic Scholar elision note in JSON and Markdown. A real failure reply names Semantic Scholar.
