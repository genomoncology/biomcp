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
