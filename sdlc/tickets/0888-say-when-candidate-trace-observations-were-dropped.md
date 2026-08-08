---
flow: build
priority: 4
---
# Say when candidate-trace observations were dropped

## Done when

When `candidate_trace` omits route observations because the merged set
exceeded `ITEM_WORK_LIMIT`, the output says so and says how many were
dropped. A trace that reports nothing omitted really had nothing
omitted.

## Why here, why now

Same class as the article-asset issues already in this backlog: a
truncation that reads as a complete answer.

## The finding

Raised under March and carried over when BioMCP moved to the sdlc
factory. The text below is as filed.

`candidate_trace` is capped at `ITEM_WORK_LIMIT`, but merged candidates can retain
more than that many route observations. The current debug schema states that it is
bounded without saying when observations were omitted.

## Detail

A later visible result can lack a trace row if earlier candidates have multiple
route observations. That makes the live canary conservatively fail its
route-attribution gate, but leaves an operator unable to distinguish a genuine
pipeline loss from diagnostic-trace truncation.

## Suggested action

Choose and document a deterministic trace selection policy (prefer visible and
filtered candidates) plus an omission indicator, or revise the trace contract to
make its sampling semantics explicit. Add a frozen fixture with more observations
than the bound and a behavioral `spec` assertion for the selected policy and
omission signal.
