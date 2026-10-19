---
priority: 3
---

# 2163 — Give fixture scenarios owned identifiers

Status: OPEN.

Milestone: 0.9.4

## Outcome

Recorded article fixture scenarios have named owners and identifiers, so a fixture repair cannot silently change an unrelated scenario's provider replies.

- Starts from: the article fixtures and test consumers involved in the 2131/2150 regressions; inventory actual shared identifiers before choosing a migration batch.
- Keeps: source facts, provider behavior, product identity rules and existing assertion strength.
- Changes: claims `spec/fixtures/setup-article-fulltext-source-fixture.sh` `tests/rmcp_client_contract.rs` `CHANGELOG.md`. Narrow the first batch's consumer claims after the inventory, before code; request an expansion for other files.
- Proof: a recorded outside-in case demonstrates the inventoried collision before migration, then unrelated named scenarios continue to return their own replies when one is changed. Run affected cases through the build job.
- Defers: implementation until the next release. A narrow fixture repair needed for a current-release product fix stays in that product ticket; this ticket does not block 0.9.3.

PM decision: 2026-10-17. Ian deferred the broad scenario refactor to keep current-release prevention fixes small.

## Progress

- 2026-10-19 started
