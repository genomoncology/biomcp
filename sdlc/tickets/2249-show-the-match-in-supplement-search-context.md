---
priority: 2
---

# 2249 — Show the match in supplement search context

Status: OPEN.

Milestone: 0.9.4

## Outcome

Each `article supplement-search` hit's context includes the matched string with bounded text around it.

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
