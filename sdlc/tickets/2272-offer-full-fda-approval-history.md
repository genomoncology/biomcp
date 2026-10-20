---
priority: 3
---

# 2272 — Offer full FDA approval history

Status: OPEN.

Milestone: 0.9.5

Reviews: revision 287285f, accept

## Outcome

`get drug ... approval-history` shows the application's full provider-returned Drugs@FDA submission history with source class, date and document links. The opt-in section reuses the existing section registry; approvals, regulatory and all retain compact output, and all does not include full history.

Changes: `src/sources/openfda.rs`, `architecture/technical/source-integration.md`, `src/entities/source_state_registry.rs`, `src/render/provenance.rs`, `src/render/markdown/drug.rs`, `src/transform/drug.rs`, `src/entities/drug/get/tests.rs`, `testdata/sources/capture-receipts.json`, `src/entities/drug/mod.rs`, `src/entities/drug/get.rs`, `src/entities/drug/approvals.rs`, `src/entities/drug/metadata.rs`, `src/entities/drug/metadata/tests.rs`, `src/render/markdown/drug_regulatory.rs`, `src/cli/list/clinical.rs`, `src/cli/commands/selectors.rs`, `docs/reference/drug-approvals-and-ae-summary.md`, `spec/entity/drug.md`, `tests/test_drug_regulatory_replay.py`, `testdata/sources/openfda/*`, `CHANGELOG.md`.

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

## Reproduction on current main

Before product edits, M5 job 2272/1792505332201-43094 ran `pm job run --machine m5 2272 e0d84c4293051b7f466bfcc58a68e2b739aace10 -- env BIOMCP_BIN=/Users/ian/workspace/worktrees/biomcp-2258-job/target/debug/biomcp /Users/ian/workspace/worktrees/biomcp-2258-job/.venv/bin/python -m pytest -q /tmp/biomcp2272-proof/tests/test_drug_regulatory_replay.py -k approval_history`. It failed one case with twelve deselected: the compact replay first returned eight submissions without class, then the actual main CLI exited 2 with invalid_argument, Unknown section approval-history for drug. The authorized keyless BLA125514 capture succeeded once with HTTP 200 and 127 submissions; fixture and exact request metadata are retained in testdata/sources/openfda/drugsfda_keytruda_20261020.json and its .request.json. These are observed capture counts, not hardcoded product limits.

## Reviewed representation

The new opt-in section uses the existing approvals JSON payload with every provider-returned row, source class and document data. It does not add a second top-level history field or change compact projections. Registry and provenance additions remain isolated on this branch while release repair 2287 changes the protein credit contract; this feature will be rebased and reviewed after that repair and cannot land before the tag. The approved successful FDA capture uses the existing shared capture-receipts format rather than a parallel request-metadata mechanism.

## Ready review evidence

Implementation 4213dd380f4eaccc5f25fd524e158f26eb9be61e passed full mechanical lint, 218 focused Rust cases and all thirteen regulatory CLI replays. Final focused replay log is 2272/1792506264770-34203. The developer disclosed two local shortage-fixture routing/envelope failures, corrected only those fixture details, and reused the unchanged product source Rust proof. No second provider request occurred. After the developer stopped, the coordinator merged main 6e664cfa20164b58339882d92e2a13c752d222fa and preserved both the accepted protein UniProt table correction and this new history row. Fresh review covers that integration; no landing before the tag.

Keep: experiments/2102-biomcp-2258-final-live-replay — retain fda-history/2272 raw job evidence for accepted source287285f24321fb960b6694f563af0141ac3d9b9f through its future 0.9.5 landing. The completed M5 job checkout was removed after retaining all raw runs; the accepted source branch remains held until after the 0.9.4 tag. No source or main integration changed during cleanup.
