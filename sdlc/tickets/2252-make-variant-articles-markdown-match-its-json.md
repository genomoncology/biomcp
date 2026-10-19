---
priority: 2
---

# 2252 — Make variant articles Markdown match its JSON

Status: OPEN.

Milestone: 0.9.4

## Outcome

The Markdown source plan and source status of `biomcp variant articles` report the same sources and states as its JSON.

## Evidence

Reproduced on 0.9.3: `biomcp variant articles "BRAF V600E" --limit 5` prints "Source plan: candidates [PubTator3, Europe PMC, Semantic Scholar]; enrichment [Semantic Scholar]" and "Semantic Scholar: disabled" with `S2_API_KEY` set, while JSON `source_status` shows LitVar2, PubMed and Semantic Scholar routes ok and rows list PubMed and Semantic Scholar.

## Keeps

Render Markdown from the same source-status data as JSON; no second computation.

## Proof

A replay test renders both formats for one run and checks they name the same sources and states, with and without the Semantic Scholar key.
