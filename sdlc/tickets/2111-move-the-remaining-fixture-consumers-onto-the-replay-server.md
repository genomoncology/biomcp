---
flow: build
priority: 3
deps: [2080, 2079]
---

# 2111: Move the remaining fixture consumers onto the replay server

Status: OPEN.

Milestone: 0.9.3

## Evidence

- 2080 deleted the fixture orchestration from the spec path but kept five fixture scripts plus supervisor and run helpers (article-fulltext-source, study, ctgov-intervention-alias, provider-contract, complexportal; live canaries) because live tests still consume them: tests/article_asset_terminal.rs, crates/biomcp-mcp-contract-client, tests/conftest.py, tools/check-png-artifact, test_ctgov_trial_search_detail_reuse.py, test_provider_contract_fixture.py, test_complexportal_fixture_contract.py, verify's cli.md.

## Work

- Record each consumer's replies under testdata/replay and move the test onto the replay server; delete each fixture script as its last consumer moves.

## Acceptance

1. No test outside verify-lane live canaries starts a fixture server; the five scripts and their helpers are gone.
