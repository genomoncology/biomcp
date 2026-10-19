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
