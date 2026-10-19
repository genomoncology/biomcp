---
priority: 1
---

# 2250 — Mark supplement search incomplete when files are unreachable

Status: OPEN.

Milestone: 0.9.4

## Outcome

`article supplement-search` lists every supplement file it could not read and marks the search incomplete, so "no hits" never reads as "not present".

## Evidence

Reproduced on 0.9.3: `biomcp --json article supplement-search 38368708 I448T` returns success with no files and no hits, although `get article 38368708 assets --asset-view coverage` lists a blocked DOCX, and searching that file alone errors. The same happens for five unreachable files of 27397505.

## Keeps

Keep successful hits and the deadline. Use the same unavailable-file wording as the asset coverage view.

## Proof

Replay tests for both papers show the unreachable files named and the result marked incomplete in JSON and Markdown.

## Coordinator study brief

Restate the approved outcome, reproduce on current main before source edits, and report the exact command/output, proposed precise source/test paths and outside-in failing case. Reuse qualified unchanged current-main CLI when possible; all execution goes through the configured build job. Preserve exact raw source responses and invocation in the next numbered three-digit experiment on the executing host, with OWNER biomcp/2250. No live rerun if an existing deterministic replay already reproduces the same failure; any authored controls must be explicitly identified, not called captured source. Read source redistribution standards before publishing new captures. No product or test claim is granted yet. No source edits, WIP pushes, child agents, mail or status changes during this study. Stop after bounded reproduction and proposal; do not widen the ticket.

## Granted implementation brief

- Changes: `src/entities/article/supplement_search.rs` `src/entities/article/mod.rs` `src/cli/article/dispatch.rs` `src/render/markdown/article.rs` `src/render/markdown/mod.rs` `tests/article_asset_terminal.rs` `spec/entity/article.md` `src/cli/list/literature.rs` `CHANGELOG.md` `sdlc/tickets/2250-mark-supplement-search-incomplete-when-files-are-unreachable.md`.

Restated outcome: retain readable supplement hits and deadlines, list every named unreadable supplement with the same outcome wording as asset coverage, and mark the search incomplete in JSON and Markdown. Minimal Markdown support for this command replaces its obsolete JSON-only refusal. No unrelated article surface changes.

Current-main8cf9c3be7 deterministic job1792424084240-92557: existing22663023 coverage named NIHMS265402-supplement-Supplementary_Tables.xls with pmc_proof_of_work; supplement-search I448T returned exit0/files[]/hits[]; Markdown refused JSON-only. Evidence790-biomcp-unreadable-supplement-search retains exact commands/outputs, recorded proof-of-work input and authored fixture identifiers. This reproduced the same class without live target reruns. PM supplied existing source evidence for38368708 and27397505; copy relevant raw into790 without modifying its owner source folder, inspect redistribution standards, and add the named-paper replay proof truthfully. Missing bytes must be reported rather than silently skipped. Never invent a checksum for an unreadable file. Keep successful-hit/parser-limit coverage and add multi-file coverage.2249 follows this accepted candidate because both touch the same search module.

Write failing outside-in tests first, apply the minimal repair, run affected tests and mechanical checks through the configured build job. Preserve private patch and per-file hashes; verify published source matches the tested manifest. No WIP pushes, merges, ticket/status edits, mail or child agents. Push once review-ready and stop after reporting.

Keep: experiments/790-biomcp-unreadable-supplement-search — Retain cited raw proof until release evidence is explicitly released.
