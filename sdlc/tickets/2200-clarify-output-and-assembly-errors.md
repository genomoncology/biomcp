---
priority: 3
---

# 2200 — Clarify output and assembly errors

Status: OPEN.

Milestone: 0.9.4

## Outcome

Error messages say what went wrong.

- Evidence: release review of 7025e117: `--output ./` gives `Device or resource busy (os error 16)` and an existing directory gives `Not a directory` instead of saying the output is a directory. A not-found variant with `--assembly hg38` embeds `(attempted GRCh38; upstream HTTP 404)` inside the quoted ID and suggests `biomcp search variant` with no arguments. The FDA orphan health row gives no reason now that the site no longer redirects. The GWAS health probe gets HTTP 410 while search works (also in v0.9.2).
- Proof: outside-in cases for each message.

## Progress

- 2026-10-18 started

## Development brief

Restated outcome: directory output destinations identify the file-path error; variant misses quote only the requested identifier, put attempted build and upstream context outside it, and provide a runnable search command containing the input. Health failures disclose the reason observed.

Changes: `src/utils/download.rs`, `src/entities/variant/get.rs`, `src/entities/variant/get/**`, `src/sources/fda_orphan.rs`, `src/cli/health/runner.rs`, `src/cli/health/http.rs`, `src/cli/health/catalog.rs`, `tests/article_asset_terminal.rs`, `tests/disable_sources_cli.rs`, `spec/entity/variant.md`, `CHANGELOG.md` and this ticket. Add an exact claim before any further file is needed. Do not redesign the generic error envelope or provider retrieval.

Study found output-directory rename errors and variant miss formatting still present.2088 already explains FDA redirects, but the nonredirect Failed path drops HTTP, parse or transport reasons. Preserve redirect behavior and fix the missing failed-state explanation. GWAS health probes the retired rsID resource while association search uses the supported v2 route. Authorize the health probe to use the existing supported v2 association-search route and describe the search behavior it actually checks; leave variant-by-rsID retrieval unchanged. This follows the existing ticket evidence and changes no clinical interpretation.

Before product edits, reproduce with isolated recorded loopback fixtures using current main: `biomcp get article 22663011 asset traces-s1.csv --output ./` and an existing-directory destination; JSON and human `biomcp get variant --assembly hg38 chr7:g.140453136A>T`; FDA nonredirect failure plus retained redirect/valid-table controls; GWAS health paired with `biomcp --json --no-cache search gwas -g TCF7L2 --limit 1`. Extend existing acceptance cases first and record exact commands/output. No live providers or new capture are authorized.

## Current-main reproduction

Job2200/1792359750841-38394 ran acceptance-only overlays against qualifieddd365f45a, binarySHA2561bdc92ee5b1a1a29c9fa7122dcec9a100ed648dcdf9e8e27fa61ec550deb646c. All4selected cases failed for the intended behavior; exit100. Command: `cargo nextest run --no-default-features --test article_asset_terminal --test disable_sources_cli --no-fail-fast --nocapture -E 'test(asset_output_directory_destinations_name_the_file_path_error) | test(variant_miss_keeps_assembly_outside_the_quoted_identifier) | test(fda_nonredirect_health_failures_explain_the_observed_reason) | test(gwas_health_checks_the_supported_association_search)'`. Only needed Rust acceptance harnesses compiled, with the job-only CLI path set to the frozen qualified baseline. No product edit or live provider request.

The article output ./ case exits1 with IO Invalid argument errno22; an existing directory exits1 with IO Is a directory errno21. Both omit the destination, superseding the older platform-specific errno examples. Human variant miss quotes the identifier together with attemptedGRCh38/upstreamHTTP404 and suggests argument-free biomcp search variant. Its assertion stopped before the JSON iteration, so JSON baseline execution is not claimed. FDA503 and malformed200 return error/latency without a reason; redirect302 still explains the blocked redirect and a valid recorded table succeeds. Recorded GWAS v2 search succeeds while health hits the retired route and reports410.

Proceed with the scoped shared-source fixes and retained controls. The generic error envelope and variant GWAS retrieval remain unchanged. Run affected cases, including the previously unrun variant JSON iteration, then format/lint/quality checks before the single ready push and fresh review.
