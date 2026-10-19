---
priority: 1
---

# 2273 — Keep comparison symbols in article abstracts

Status: OPEN.

Milestone: 0.9.4

## Outcome

`get article` abstracts keep every character the publisher's PubMed record holds, including comparison symbols such as ≧, ≥ and ≤, so no threshold is dropped or inverted.

## Evidence

Research (R17, verified against PubMed EFetch): `biomcp --json get article 33311791` renders "Elderly patients ( 65 years, n=91)". PubMed EFetch holds "≧65 years", but PubTator3's biocjson has already dropped the symbol, and BioMCP passes PubTator3's abstract through. A dropped comparison symbol can invert a threshold in clinical text.

## Keeps

Prefer PubMed's abstract text when both are available, or detect characters PubTator3 lost against PubMed's text and use PubMed's. Keep PubTator3's annotations. Coordinate with 2268, which backfills empty abstracts.

## Proof

A replay test for 33311791 shows "≧65 years". A test with symbols ≥, ≤, <, >, ± and µ in the PubMed reply keeps them all.
