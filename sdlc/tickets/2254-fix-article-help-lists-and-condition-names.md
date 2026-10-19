---
priority: 3
---

# 2254 — Fix article help, command lists and condition names

Status: OPEN.

Milestone: 0.9.4

## Outcome

Changes: `spec/entity/article.md`, `src/cli/article/mod.rs`, `src/cli/variant/mod.rs`, `src/cli/list/literature.rs`, `src/cli/list/molecular.rs`, `src/cli/list/catalog.rs`, `src/entities/article/assets.rs`, `src/entities/article/mod.rs`, `src/cli/system/dispatch.rs`, `src/cli/article/dispatch.rs`, `src/render/markdown/article.rs`, `src/render/markdown/related/article_support.rs`, `tests/article_batch_outcome.rs`, `tests/rmcp_client_contract.rs`, `tests/article_asset_terminal.rs`, `docs/user-guide/supplementary-materials.md`, `docs/reference/cli-reference.md`, `CHANGELOG.md`, `sdlc/tickets/2251-report-publisher-elided-references-as-empty.md`, `sdlc/tickets/2252-make-variant-articles-markdown-match-its-json.md`, `sdlc/tickets/2253-document-full-text-access-groups-and-reason-codes.md`, `sdlc/tickets/2254-fix-article-help-lists-and-condition-names.md`.

Article and variant help, `biomcp list`, and the CLI reference agree with what the commands accept, and one blocked-download condition has one name.

## Evidence

Research audit of 0.9.3:
1. `article supplement-search` help offers `-j, --json` but the command is JSON-only and exits 2 without it; a stray `\` line precedes EXAMPLES.
2. `variant articles --input` needs `--json`, and its help does not say so.
3. `--out`, `--output`, `--outline` and `--lines` appear in help but not in `biomcp list article`; `--out` and `--output` are in no doc. `--confirmed-only` is missing from `biomcp list variant`. `supplement-search` is missing from the Commands list of `biomcp list article` and from `cli-reference.md`.
4. Asset manifests report `pmc_proof_of_work` while downloading the same file fails with `ncbi_interstitial` (`src/entities/article/assets.rs` near line 1892), which no doc mentions.
5. A full-text miss on a paper with an open PDF (22663011, PDF status BRONZE) suggests annotations, tldr and indexing but not `--pdf`, although `fulltext --pdf` succeeds.
6. Compact batch output nests `# Article Batch (1)` under `## PMID — ok`.

## Keeps

Prefer generating lists from the command definitions over hand edits where a list already is generated. Keep the old condition name accepted where it is part of JSON output, and document the mapping.

## Proof

A test compares `biomcp list` entries with the command definitions. Tests cover the `--pdf` suggestion, one condition name across manifest and download, and the batch heading level.

## Progress

- 2026-10-19 started
