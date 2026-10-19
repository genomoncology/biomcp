---
priority: 1
---

# 2248 — Keep preprint full text from panicking

Status: OPEN.

Milestone: 0.9.4

## Outcome

`get article` full text for a preprint DOI returns a structured answer, the text or a named reason, and never panics. A batch keeps every unrelated item when one fails.

## Evidence

Reproduced on 0.9.3: `biomcp --json --no-cache get article 10.1101/2022.07.18.22276644 fulltext` panics at a provenance invariant (`provenance.rs:115`) and writes zero JSON bytes. `batch article 14735200,25922291,18178632,18509008,12752119,20301425,10.1101/2022.07.18.22276644 --mode detail --sections fulltext` loses all seven items. The preprint server's API has the JATS body, and `--pdf` returns real text.

## Keeps

Fix the invariant's cause, not just the symptom. Keep provenance accurate for every source.

## Proof

Reproduce on current main first. A replay test returns the preprint's text with its provenance, and the batch returns all seven items with the preprint's outcome named.

## Coordinator study brief

Restate the approved outcome, reproduce on current main before source edits, and report the exact command/output, proposed precise source/test paths and outside-in failing case. Reuse qualified unchanged current-main CLI when possible; all execution goes through the configured build job. Preserve exact raw source responses and invocation in the next numbered three-digit experiment on the executing host, with OWNER biomcp/2248. No live rerun if an existing deterministic replay already reproduces the same failure; any authored controls must be explicitly identified, not called captured source. Read source redistribution standards before publishing new captures. No product or test claim is granted yet. No source edits, WIP pushes, child agents, mail or status changes during this study. Stop after bounded reproduction and proposal; do not widen the ticket.
