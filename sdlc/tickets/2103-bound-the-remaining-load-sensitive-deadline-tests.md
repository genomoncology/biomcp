---
flow: build
priority: 2
deps: [2102]
---

# 2103: Bound the remaining load-sensitive deadline tests

Status: OPEN.

Milestone: 0.9.3

## Evidence

Filed 2026-10-15 from lanes 2066 and 2067 under the PM's two-jobs doctrine (a timing test that fails only with concurrent jobs is a test defect). Each fails under two concurrent Yellow jobs and passes otherwise; ticket 2102 already covers the rmcp contract deadline test, these are the rest observed the same day:

- add_clinvar_admits_rate_limiting_when_a_sustained_429_outlives_the_deadline (ClinVar 429 deadline bound).
- test_rich_cli_json_and_markdown_are_well_formed (60s subprocess deadline).
- test_rich_raw_mcp_matches_cli_byte_for_byte (60s subprocess deadline).
- test_rich_malformed_provider_page_is_an_mcp_error_not_an_empty_page (30s MCP select deadline).

## Work

Bound or shed the load sensitivity of each: raise the deadline for contract purposes, split the converging pair, or make the subprocess fixture deterministic. Each must pass with two jobs running.

## Acceptance

1. All four pass in a full suite while another Yellow job runs concurrently, twice.

## Added 2026-10-15 (coordinator)

- New instance from 2071's lane: `tests/test_output_footprint_benchmark.py::test_offline_corpus_is_deterministic_and_reports_real_token_counts` - `article_search_compact` hit its 60s deadline only under lane-build contention on Yellow (load 8+); passes in CI at the same tip. Same class, this ticket's home.
- Also from 2100's lane: `human_mcp_command_dispatches_to_provider_once` and the full-suite timeout of `raw_mcp_rejects_a_command_the_surface_refuses` - load-only failures, pass isolated with the export. Same class, this ticket's home.

