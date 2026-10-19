---
priority: 2
---

# 2249 — Show the match in supplement search context

Status: OPEN.

Milestone: 0.9.4

## Outcome

Each `article supplement-search` hit's context includes the matched string with bounded text around it.

- Changes: `CHANGELOG.md` `sdlc/tickets/2245-name-the-cgi-sections-source.md` `sdlc/tickets/2246-query-every-coding-alias-or-report-dropped-forms.md` `sdlc/tickets/2247-search-previous-gene-symbols-in-variant-articles.md` `sdlc/tickets/2248-keep-preprint-fulltext-from-panicking.md` `sdlc/tickets/2249-show-the-match-in-supplement-search-context.md` `sdlc/tickets/2250-mark-supplement-search-incomplete-when-files-are-unreachable.md` `spec/entity/article.md` `spec/fixtures/setup-article-fulltext-source-fixture.sh` `src/cli/article/dispatch.rs` `src/cli/list/literature.rs` `src/entities/article/mod.rs` `src/entities/article/supplement_search.rs` `src/render/markdown/article.rs` `src/render/markdown/mod.rs` `tests/article_asset_terminal.rs`.


## Evidence

Reproduced on 0.9.3: `biomcp --json article supplement-search 24451234 R1436Q` finds the right page-21 hit, but its context omits the matched string.

## Keeps

Keep hit location and ranking. Bound the context length.

## Proof

A fixture test shows the context containing R1436Q for that supplement page, and a match near the start or end of a page stays within bounds.

## Coordinator study brief

Restate the approved outcome, reproduce on current main before source edits, and report the exact command/output, proposed precise source/test paths and outside-in failing case. Reuse qualified unchanged current-main CLI when possible; all execution goes through the configured build job. Preserve exact raw source responses and invocation in the next numbered three-digit experiment on the executing host, with OWNER biomcp/2249. No live rerun if an existing deterministic replay already reproduces the same failure; any authored controls must be explicitly identified, not called captured source. Read source redistribution standards before publishing new captures. No product or test claim is granted yet. No source edits, WIP pushes, child agents, mail or status changes during this study. Stop after bounded reproduction and proposal; do not widen the ticket.

Depends on: 2250

## Bounded study and dependency queue

Current-main8cf deterministic job1792424213233-98997 seeded the existing local extraction fixture with explicitly authored prefix×100,F1761I,suffix×100. One Sheet1row2780 hit had400bytecontext and omitted the matched query; no livecalls. Fix context around the actual matching span with Unicode-safe boundaries while preserving hitselection/order/locators and equivalent-protein spelling rules. Retain the entire matched spelling when it exceeds the ordinarycontextbudget, bounded by the existing query limit; ellipses count within the effectivecontextbound. Reuse existing outside-in case with beginning/end/late/Unicode/equivalent-spelling controls. No newmatching semantics. Source paths will be supplement_search.rs,tests/article_asset_terminal.rs,CHANGELOG; exactclaim is granted after dependency candidate is adopted.

Implementation waits for the accepted2250candidate because these changes share source files. The coordinator will adopt that candidate; no independent overlapping source writer is authorized.

Keep: experiments/793-biomcp-2249-context-study — Retain cited raw proof until release evidence is explicitly released.

## Granted context implementation

The coordinator adopted accepted2250source at383fc9c28ba540004db92c0d1692de01c76354f7 plus its review record. Implement only the remaining context bug in the three declared product files. Reuse793authoredred/control fixture; no live rerun or newmatching identity rules. Return the actual matched span from the existing matcher and select Unicode-safe surrounding context, preserving hit selection/order/locators and equivalent protein spellings. Beginning/end/late/Unicode/equivalent-spelling cases and a match longer than the ordinarycontextbudget must retain the complete matched string. Count ellipses inside the effectivebound; do not emit an arbitrarily long wholeblock. Preserve2250unreadablefiles/incompleteJSON/Markdown/readablehits/deadline proof.

Write outside-in failing cases, then minimal fix, run only affected cases/mechanicals through configuredbuildjob. Retainpatch/manifests/raw in793, comparefinalpublishedblobs, publishoncegreen/noWIP, no merges/children/status/mail/ticketedits.

## Focused context repair proof

Source50bd4e62e4ed5adf53e6f36b7383bc8a391aba77 changes only the three declared productfiles after accepted2250dependency. The existing matcher returns its accepted span and context uses Unicode-safe boundaries, with ellipses inside its bytebound; a longer accepted match retains its complete spelling rather than an arbitrary wholeblock. Selection/order/locators/protein-candidatepriority remain unchanged.

Red1792425793688-38488 reproduced the late/end context omission through the real CLI using explicitly authored cachedtext. Green1792425865590-19960 passed eight affected tests, including beginning/end/late/Unicode/both equivalent-protein directions/longmatch/Markdown, both accepted supplement examples,2250unavailable-file/mixedreadablehits and the extraction-worker deadline cleanup. Formatting, warnings-denied Clippy, quality ratchet and whitespace passed. Published blobs match the retained tested-file manifest. Rawlog/checkscript/patches/manifests remain in experiments/793-biomcp-2249-context-study; no livecapture or matching-rule change.
