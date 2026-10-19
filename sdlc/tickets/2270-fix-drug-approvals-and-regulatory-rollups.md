---
priority: 2
---

# 2270 — Fix drug approvals and regulatory rollups

Status: OPEN.

Milestone: 0.9.4

## Outcome

Drug approvals find Drugs@FDA records filed under active ingredient or brand name, regulatory section status rolls up its parts truthfully, safety labels show their effective date, and an expected empty target answer prints no warning.

## Evidence

Research ran released 0.9.3 on benchmark questions and paper fetches and verified each finding against the provider's official API; raw outputs are in research's store and may be copied as fixtures (ask the PM for the path). R10: `biomcp --json batch drug "daratumumab,belantamab mafodotin" --sections approvals` returns empty approvals while overviews say approved; Drugs@FDA finds daratumumab under products.active_ingredients.name (BLA761036, BLA761145) and BLENREP under products.brand_name (BLA761440); openfda.generic_name finds none. R11: `biomcp --json get drug zotiraciclib regulatory --region us` reports regulatory as data while approvals is empty and orphan designations unavailable. R16: `get drug lorcaserin safety` shows a 2018 label (effective_time 20180429) with nothing marking its age. R15: `get drug --name "trichostatin A" targets` warns "missing field; degrading response" when OpenTargets returns mechanismsOfAction:null and no source holds a target.

## Keeps

Keep the drug identity guards; a fallback match must be the same drug. Show the label date, not an inferred withdrawal.

## Proof

Replay tests for each of the four cases.
