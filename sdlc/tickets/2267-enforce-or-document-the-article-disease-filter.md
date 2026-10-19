---
priority: 2
---

# 2267 — Enforce or document the article disease filter

Status: OPEN.

Milestone: 0.9.4

## Outcome

`search article -d` either filters every source's rows to the disease or is documented and labeled as ranking only; help and output agree with the behavior.

## Evidence

Research ran released 0.9.3 on benchmark questions and paper fetches and verified each finding against the provider's official API; raw outputs are in research's store and may be copied as fixtures (ask the PM for the path). R6: `biomcp search article -d "rectal cancer" -k "locally advanced initial staging routine PET CT guidelines" --full --json --limit 5` returns PMID 39118875 (lung cancer, from Semantic Scholar) in third place; its PubMed record contains no "rect". Help calls `-d` a filter.

## Keeps

Prefer enforcing it as a filter on sources that cannot filter upstream, using the disease and its synonyms, if that keeps results useful; otherwise rename it in help as a ranking term. Keep other sources' behavior.

## Proof

A replay test for the rectal cancer query shows the lung cancer row removed, or labeled as unfiltered with help saying so.
