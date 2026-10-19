---
priority: 3
---

# 2272 — Offer full FDA approval history

Status: OPEN.

Milestone: 0.9.5

## Outcome

`get drug ... approvals` can show an application's full Drugs@FDA submission history with class, date and document links.

## Evidence

Research ran released 0.9.3 on benchmark questions and paper fetches and verified each finding against the provider's official API; raw outputs are in research's store and may be copied as fixtures (ask the PM for the path). R14 (feature request): `biomcp --json get drug Keytruda approvals` shows 8 recent submissions per application with no class or documents; Drugs@FDA for BLA125514 holds 127 submissions, 108 classed EFFICACY, each with a date and approval-letter and label URLs.

## Keeps

Default output stays compact; the full history is opt-in. Feature, not a bug, so it sits in 0.9.5.

## Proof

A replay test for BLA125514 shows the full classed history behind the opt-in flag.
