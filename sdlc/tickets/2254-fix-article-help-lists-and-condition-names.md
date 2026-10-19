---
priority: 3
---

# 2254 — Fix article help, command lists and condition names

Status: OPEN.

Milestone: 0.9.4

Reviews: revision e3a388d, accept

Reviews: revision 2a1c178, accept

Reviews: revision e55485a, accept

Reviews: revision 1bc0ced, accept

## Outcome

Changes: `tools/dead-code-exceptions.json`, `src/render/markdown/article/tests.rs`, `src/render/markdown/mod.rs`, `spec/entity/article.md`, `src/cli/article/mod.rs`, `src/cli/variant/mod.rs`, `src/cli/list/literature.rs`, `src/cli/list/molecular.rs`, `src/cli/list/catalog.rs`, `src/entities/article/assets.rs`, `src/entities/article/mod.rs`, `src/cli/system/dispatch.rs`, `src/cli/article/dispatch.rs`, `src/render/markdown/article.rs`, `src/render/markdown/related/article_support.rs`, `tests/article_batch_outcome.rs`, `tests/rmcp_client_contract.rs`, `tests/article_asset_terminal.rs`, `docs/user-guide/supplementary-materials.md`, `docs/user-guide/cli-reference.md`, `CHANGELOG.md`, `sdlc/tickets/2251-report-publisher-elided-references-as-empty.md`, `sdlc/tickets/2252-make-variant-articles-markdown-match-its-json.md`, `sdlc/tickets/2253-document-full-text-access-groups-and-reason-codes.md`, `sdlc/tickets/2254-fix-article-help-lists-and-condition-names.md`.

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

## Verification

The focused baseline failed all four claimed CLI regressions. Final job 2254/1792427716050-37286 passed the owning CLI and MCP cases, formatting, all-target lint and relevant boundary checks. Earlier jobs 1792427613519-89627 and 1792427546828-74059 passed the legacy condition alias, existing proof-of-work case and PDF failure behavior. Compact output explicitly checks one batch heading and correctly nested item headings. Changed full-text examples are labeled live; no live provider lookup or full gate ran. The supplement-search Markdown behavior depends on the accepted 2250 change.

Keep: experiments/796-biomcp-2254-article-consistency — retain baseline and green raw output, checked patch and commands.

Review preserved standalone batch rendering and made nested item context explicit in both dispatch callers. The standalone regression failed before the fix; final mechanical job 2254/1792428072930-17572 passed after focused renderer, CLI and MCP checks. Source accepted at e3a388daab68a1ac42a81b5d0bebdf6330c47990. Raw remains in the retained experiment.

Review preserved standalone batch rendering and made nested item context explicit in both dispatch callers. The standalone regression failed before the fix; final mechanical job 2254/1792428072930-17572 passed after focused renderer, CLI and MCP checks. Source accepted at e3a388daab68a1ac42a81b5d0bebdf6330c47990. Raw remains in the retained experiment.

## Batch quality registration

The shared candidate stopped before tests because the reviewed standalone article_batch_markdown allowance was missing from the explicit exception inventory. Preserve the re-exported standalone contract and separate nested renderer; register its exact existing reason and removal condition in the owning inventory. Keep the failed gate and owning audit result with the retained article-consistency evidence.
