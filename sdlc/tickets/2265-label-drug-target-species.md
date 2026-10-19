---
priority: 2
---

# 2265 — Label drug target species

Status: OPEN.

Milestone: 0.9.4

## Outcome

`get drug ... targets` labels each target's organism, lists human targets first, and suggests `get gene` only for human targets.

## Evidence

Research ran released 0.9.3 on benchmark questions and paper fetches and verified each finding against the provider's official API; raw outputs are in research's store and may be copied as fixtures (ask the PM for the path). R2: `biomcp --json get drug bortezomib targets` lists PF3D7_1011400 first with no organism and suggests `get gene PF3D7_1011400`; MyChem's gtopdb interaction_targets give species "Plasmodium falciparum 3D7".

## Keeps

Non-human targets stay listed, labeled. Species comes from the source record, never inferred.

## Proof

A replay test for bortezomib shows the species label, human targets first, and no gene suggestion for the Plasmodium target.
