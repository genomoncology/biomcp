# 2166 — accept-changelog-entries-for-landed-ticket-batches

Status: OPEN.

Milestone: 0.9.3

## Outcome

Accept a current-release changelog entry for a batch already landed in this release even when the ticket's unfinished work moves to a later milestone. Keep rejecting missing tickets and unlanded future-release work. Do not remove the valid replay/golden entry or change its ticket's milestone.

Priority: 1.

## Changes

- tests/test_docs_changelog_refresh.py
- CHANGELOG.md

## Implementation brief

Current main 6a7b7b120 fails test_dev_cycle_section_carries_its_own_tickets because a legitimate landed batch belongs to a ticket whose remaining work moved to 0.9.4. Job 1792243618201-65309 reproduced this unchanged guard at line 600. Use existing landing history since the previous release rather than a hardcoded ticket exception. First reproduce with a focused current-main job and add a negative unlanded-future case. Run only the relevant Python contracts through the build job system; no Rust build is needed.
