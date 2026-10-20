---
priority: 2
---

# 2303 — Share one Rust test HTTP server

Status: OPEN.

Milestone: 0.9.4

Depends on: 2301

## Outcome

Rust tests that need a local HTTP server use one shared test server that handles WouldBlock and connection resets, shuts down without panicking in Drop, and reports its own failures as test errors. Hand-written per-test socket servers move to it.

- Proof: The shared server passes a parallel stress run with clients that disconnect early; a grep finds no other hand-rolled TcpListener test server.

Changes: `crates/biomcp-mcp-contract-client/src/lib.rs`, `crates/biomcp-mcp-contract-client/src/test_http.rs`, `crates/biomcp-mcp-contract-client/src/test_http/tests.rs`, `src/cli/article/tests/entities_disease_links.rs`, `src/cli/article/tests/outage.rs`, `src/cli/discover.rs`, `src/cli/health/tests/http.rs`, `src/cli/system/batch.rs`, `src/cli/system/tests/dev2_contracts.rs`, `src/cli/tests/printed_card_commands.rs`, `src/cli/update/tests.rs`, `src/entities/article/assets.rs`, `src/entities/article/backends/tests.rs`, `src/entities/article/detail/tests.rs`, `src/entities/article/enrichment/tests.rs`, `src/entities/article/fulltext.rs`, `src/entities/article/graph/tests.rs`, `src/entities/article/graph/tests/admission.rs`, `src/entities/article/graph/tests/cache.rs`, `src/entities/article/search/tests/deadline.rs`, `src/entities/article/search/tests/relevance_paging.rs`, `src/entities/article/test_support.rs`, `src/entities/article/variant_search.rs`, `src/entities/author/papers.rs`, `src/entities/disease/get/tests.rs`, `src/entities/disease/resolution/paging_tests.rs`, `src/entities/disease/resolution/tests.rs`, `src/entities/drug/get/tests.rs`, `src/entities/drug/search/tests.rs`, `src/entities/drug/test_support.rs`, `src/entities/drug/test_support/brand_resolution.rs`, `src/entities/drug/test_support/ingredient_resolution.rs`, `src/entities/drug/test_support/label_choice.rs`, `src/entities/drug/test_support/product_names.rs`, `src/entities/pgx/recommendation_tests.rs`, `src/entities/trial/get/tests.rs`, `src/entities/trial/search/ctgov/abbreviation_note_tests.rs`, `src/entities/trial/search/ctgov/tests.rs`, `src/entities/trial/search/nci/tests.rs`, `src/entities/trial/test_support.rs`, `src/entities/variant/clinvar.rs`, `src/entities/variant/erepo.rs`, `src/entities/variant/get/tests.rs`, `src/entities/variant/gwas/tests.rs`, `src/entities/variant/structure/tests.rs`, `src/mcp/shell.rs`, `src/mcp/shell/tests/ticket_1120.rs`, `src/sources/cbioportal_download/tests/download.rs`, `src/sources/clingen_cspec.rs`, `src/sources/gwas/tests/day_cache.rs`, `src/sources/gwas/tests/retired_endpoint.rs`, `src/sources/hpo/tests/parsing.rs`, `src/sources/mod.rs`, `src/sources/openfda/tests/day_cache.rs`, `src/sources/orcid/tests.rs`, `src/sources/pmc_article.rs`, `src/sources/provider_url_policy.rs`, `src/sources/rate_limit.rs`, `src/sources/request_plan_transport.rs`, `src/sources/semantic_scholar/tests/construction.rs`, `src/sources/tests/provider_network.rs`, `src/sources/who_pq/tests/sync.rs`, `src/test_support.rs`, `tests/adverse_event_route_contract.rs`, `tests/article_asset_terminal/primary_archive.rs`, `tests/disable_sources_cli.rs`, `tests/json_error_contract.rs`, `tests/managed_state_permissions.rs`, `tests/rmcp_client_contract.rs`, `tests/unit/cli/variant.rs`, `tests/windows_stdio_contract.rs`, `CHANGELOG.md`, `sdlc/records/2303-shared-rust-test-http.md`.

## Evidence

The 2294 batch gate failed when a hand-written fixture server got WouldBlock and panicked in Drop. 2229 and 2260 earlier cleaned up leaked replay servers built the same ad hoc way. See the root cause record `sdlc/records/2299-root-cause-of-test-only-check-failures.md`.
