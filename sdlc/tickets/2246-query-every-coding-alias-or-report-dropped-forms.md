---
priority: 2
---

# 2246 — Query every coding alias or report the dropped forms

Status: OPEN.

Milestone: 0.9.4

## Outcome

`biomcp variant articles` sends every protein and coding form it resolved for the variant to the literature sources, or names in its output and debug plan each form it did not send and why. Other-transcript forms follow the transcript-alias decision.

## Evidence

Reproduced on 0.9.3: `biomcp --json variant articles 'ABCC8 R1437Q' --limit 5 --debug-plan --verify-identity` resolves twelve protein and coding aliases but queries ten. `c.4310G>A` and `c.4373G>A` are never sent and the omission is not reported. Europe PMC `ABCC8 AND "c.4310G>A"` returns PMID 41948559, an open-access paper that names the variant only that way. Several query slots go to other-transcript protein forms (R1353Q, R1425Q, R1458Q).

## Keeps

Follow `sdlc/decisions/2026-10-18-transcript-aliases-need-the-same-genomic-change.md`: query every supplied coding form; count an other-transcript alias as evidence only when the paper pins it to the same genomic change, otherwise label it an unverified alias match. Keep the request deadline. Clinical questions go to a clinical reviewer.

## Proof

A replay test for ABCC8 R1437Q shows both coding forms sent and PMID 41948559 returned, or the dropped forms named in the output. The debug plan lists every resolved form with sent or dropped.

## Coordinator study brief

Restate the approved outcome, reproduce on current main before source edits, and report the exact command/output, proposed precise source/test paths and outside-in failing case. Reuse qualified unchanged current-main CLI when possible; all execution goes through the configured build job. Preserve exact raw source responses and invocation in the next numbered three-digit experiment on the executing host, with OWNER biomcp/2246. No live rerun if an existing deterministic replay already reproduces the same failure; any authored controls must be explicitly identified, not called captured source. Read source redistribution standards before publishing new captures. No product or test claim is granted yet. No source edits, WIP pushes, child agents, mail or status changes during this study. Stop after bounded reproduction and proposal; do not widen the ticket.
