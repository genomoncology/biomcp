---
priority: 3
---

# 2272 — Offer full FDA approval history

Status: OPEN.

Milestone: 0.9.5

## Outcome

`get drug ... approval-history` shows the application's full provider-returned Drugs@FDA submission history with source class, date and document links. The opt-in section reuses the existing section registry; approvals, regulatory and all retain compact output, and all does not include full history.

Changes: `src/sources/openfda.rs`, `src/entities/drug/mod.rs`, `src/entities/drug/get.rs`, `src/entities/drug/approvals.rs`, `src/entities/drug/metadata.rs`, `src/entities/drug/metadata/tests.rs`, `src/render/markdown/drug_regulatory.rs`, `src/cli/list/clinical.rs`, `src/cli/commands/selectors.rs`, `docs/reference/drug-approvals-and-ae-summary.md`, `spec/entity/drug.md`, `tests/test_drug_regulatory_replay.py`, `testdata/sources/openfda/*`, `CHANGELOG.md`.

## Evidence

Research ran released 0.9.3 on benchmark questions and paper fetches and verified each finding against the provider's official API; raw outputs are in research's store and may be copied as fixtures (ask the PM for the path). R14 (feature request): `biomcp --json get drug Keytruda approvals` shows 8 recent submissions per application with no class or documents; Drugs@FDA for BLA125514 holds 127 submissions, 108 classed EFFICACY, each with a date and approval-letter and label URLs.

## Keeps

Default output stays compact; the full history is opt-in. Feature, not a bug, so it sits in 0.9.5.

## Proof

A replay test for BLA125514 shows the full classed history behind the opt-in flag.

## Progress

- 2026-10-20 started

## PM decision

2026-10-20: Use the opt-in approval-history section and leave compact approvals, regulatory and all unchanged. No retained raw full BLA125514 reply exists. One exact read-only public openFDA Drugs@FDA request for BLA125514 is approved, keyless and paced, retained with request metadata as the replay fixture. At most one retry after a transient error; no other providers. Develop and review on the branch; do not land before the 0.9.4 tag.

## Starting evidence

Base e0d84c4293051b7f466bfcc58a68e2b739aace10. Read-only source inspection finds submissions truncated to eight in metadata.rs and source class/document fields absent from the submission model. Retained research CLI output for BLA125514 is from 0.9.3 and is historical only; fresh main reproduction against the authorized capture must precede implementation. The existing imatinib capture provides the field schema. The owning replay test must fail on main for the opt-in full-history result and preserve existing compact output. Record the actual reproduction command and output before changing code.
