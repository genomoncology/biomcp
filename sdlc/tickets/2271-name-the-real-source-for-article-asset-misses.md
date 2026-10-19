---
priority: 2
---

# 2271 — Name the real source for article asset misses

Status: OPEN.

Milestone: 0.9.4

## Outcome

`get article ... assets` for a paper with no open-access asset package returns a clear outcome naming the real source, never "BioMCP source" with "Review source configuration and retry", and expected misses during a successful full-text fetch stay out of stderr or move into the JSON coverage.

## Evidence

Research ran released 0.9.3 on benchmark questions and paper fetches and verified each finding against the provider's official API; raw outputs are in research's store and may be copied as fixtures (ask the PM for the path). R12: `biomcp --json get article 26617739 assets --asset-view retrievable --asset-limit 100` returns source_unavailable from "BioMCP source" with that recovery text, while full text (PMC4637554, open_access false) succeeds from PMC HTML; 31342592 gets source_unavailable too. R13: `biomcp --json get article 26617739 fulltext --pdf --out DIR` succeeds but prints "PMC OA manifest/archive request failed", "Europe PMC supplementary request failed" and an html5ever "node with weird namespace" on stderr for every non-open-access PMC fetch.

## Keeps

Real provider failures still warn and name the provider. Same wording family as 2250's unavailable files.

## Proof

Replay tests: the 26617739 assets call names the source and the no-open-access reason; the full-text call prints no expected-miss warnings and records them in coverage.
