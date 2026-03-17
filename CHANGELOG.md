# Changelog

## 0.8.16 — 2026-03-17

- Adopted the shared `bin/lint` script and wired `make lint` / `make check`
  to the March repo convention for Rust repos.
- Refreshed the README, homepage, CLI reference, and helper docs so the public
  release docs match the current BioMCP command surface.
- Expanded `skills/SKILL.md` for article graph helpers, `enrich`, `batch`,
  chart discovery, chart flags, and updated drug interaction guidance.
- Updated the Kuva chart blog post with checked-in SVG outputs for all nine
  worked examples plus placeholder references for terminal screenshots.

## 0.8.15 — 2026-03-11

- Fixed the planning-docs CI path regression so release validation uses the
  repo-local planning fixtures by default instead of an Ian-local absolute
  path. This is the fix from PR #191 that unblocks release packaging on
  GitHub Actions.
- Refreshed the public discovery docs so `search all` is taught as the unified
  cross-entity entry point in the README and docs index. This is the docs
  alignment from PR #190.
- Breaking MCP runtime change: renamed the MCP execution tool from `shell` to
  `biomcp`; update MCP clients and demos to call `biomcp` after `tools/list`.

## 0.8.14 — 2026-03-10

- Promoted remote Streamable HTTP in newcomer docs with a dedicated getting-started page for `biomcp serve-http`.
- Added a runnable PEP 723 demo client at `demo/streamable_http_client.py` for `/mcp`, including a `tools/list` flow and `biomcp version` tool call.
- Surfaced the canonical remote HTTP routes more prominently across public docs: `/mcp`, `/health`, `/readyz`, and `/`.
- Kept `serve-sse` migration guidance visible while shipping the Streamable HTTP release/docs/demo verification package together.

## 0.8.13 — 2026-03-09

- `study survival` now reports Kaplan-Meier median survival, 1/3/5-year survival landmarks, and two-group log-rank p-values.
- Fixed survival median calculation: replaced raw follow-up median with Kaplan-Meier median survival.
- `study compare --type expression` now reports Mann-Whitney U and p-value.
- Added `study download <study_id>` and `study download --list` for local cBioPortal datahub installs.
- Hardened `study download`: no total body timeout for large archives, path-like study ID rejection, stream-to-disk download, and cleanup on failed extraction.
- Refreshed `skills/SKILL.md` with real study command usage, including `cohort`, `filter`, and `survival --endpoint` examples.

## 0.8.12 — 2026-03-07

- Added `study` subcommand: local cBioPortal study adapters with CLI query surface (`download`, `cohort`, `survival`, `compare`).
- Added `study filter`: multi-omics cross-table sample joins across mutation, CNA, expression, and clinical data.
- Added Fisher exact p-values to co-occurrence pair analysis.
- Added skill installer auto-discovery: scans existing agent config directories in priority order and supports the `.agents/skills/` cross-tool standard.
- Refreshed skill guidance and reference material: adopted the measured `SKILL.md` refresh and added drift validation for examples, schemas, and jq snippets.
- CLI quality pass: positional search arguments, multi-token input handling, variant fallback, NCT ID validation, bare-entity shortcuts, staged trial fill, article request timeout, and `--until` alias.

## 0.8.11 — 2026-03-06

- Added `expression` section to gene output (GTEx tissue-specific TPM data).
- Added `druggability` section to gene output (DGIdb drug-gene interactions and categories).
- Added `clingen` section to gene output (gene-disease validity and dosage sensitivity).
- Added evidence URLs (`_meta.evidence_urls`) to all entity output — includes Ensembl, OMIM, NCBI Gene, and UniProt links where available.
- Added `spec/` BDD documentation suite with 54 passing executable specifications.
- Unified article search: fan-out across PubTator3 and Europe PMC in parallel with PMID deduplication, source-grouped rendering, and `--source <all|pubtator|europepmc>`.
- Added infinite cache mode via `BIOMCP_CACHE_MODE=infinite` for offline/demo workflows.
- Consolidated CI into a single job; streamlined release pipeline.

## 0.8.10 — 2026-03-04

- Polished `search all` output: GWAS trait relevance filtering, clinical significance sorting, uninformative variant suppression.
- Gene-anchored article search via `GENE_PROTEIN` field for higher precision.
- Added trial search improvements (`--mutation` boolean handling and `--criteria` support).
- Added PyPI trusted publisher setup and release workflow updates.

## 0.8.9 — 2026-03-03

- Added `search all` cross-entity command — parallel fan-out across major entities (genes, variants, diseases, drugs, trials, articles, pathways, PGx, GWAS, and adverse events) with counts-first display and HATEOAS deep-dive links.

## 0.8.8 — 2026-03-02

- Output cleanup across entity renderers for consistent markdown formatting.
- Improved help discoverability and completeness across all subcommands.
- Rewrote footer output with streamlined next-step suggestions.

## 0.8.7 — 2026-02-27

- Improved help text discoverability and completeness for all CLI subcommands.

## 0.8.6 — 2026-02-27

- Fixed trial filter accuracy: age-based post-filtering, exclusion-criteria detection, and pathway source correctness.

## 0.8.5 — 2026-02-26

- Fixed `install.sh` to resolve `latest` tag to a release that has downloadable assets.
- Hardened input safety with SSRF prevention on user-supplied URLs.
- Enforced pagination and validation contracts across search endpoints.

## 0.8.4

- Simplified CI/CD workflow triggers to avoid redundant post-merge runs.
- Added runner-aware `actions/cache@v4` protoc caching in CI, docs deploy, and release workflows.
- Pinned workflow protoc installation to `28.3` with cache-aware conditional setup.

## 0.8.3

- Added inclusion-first eligibility post-filtering to reduce exclusion-only false positives in trial matching.
- Applied cryptography dependency security updates and refreshed release artifacts.
- Tightened CLI/docs consistency around trial filter behavior and output contracts.

## 0.8.2

- Added location-level facility and geo post-filtering for more accurate trial site matching.
- Embedded git tag version metadata in the binary for reliable `biomcp version` output.
- Improved pagination and result-window handling across search command families.

## 0.8.1

- Expanded clinical-grade trial filtering with ~45 new search flags and validation rules.
- Added MCP tool auto-generation to reduce drift between CLI capabilities and server tool surface.
- Upgraded help/discoverability content and list-based guidance for agent-driven workflows.

## 0.8.0

Complete rewrite from Python to Rust. Single static binary, no runtime dependencies.

### Highlights

- Single-binary CLI and MCP server — no Python, no pip, no virtual environments
- 15 biomedical data sources, unified command grammar, compact markdown output
- 14 embedded skills (guided investigation workflows)
- MCP server (stdio + SSE) with tool and resource support
- HTTP proxy (`serve-http`) for multi-worker shared rate limiting
- Production installer with SHA256 verification (5 platforms)
- Progressive disclosure: search returns summaries, get returns full detail with selectable sections
- NCBI API key support for improved rate limits
- 429 retry with Retry-After backoff

### Data sources

MyGene.info, MyVariant.info (ClinVar, gnomAD, CIViC, OncoKB), ClinicalTrials.gov,
NCI CTS API, PubMed/PubTator3, MyChem.info, Monarch/MONDO, Reactome, UniProt,
OpenFDA FAERS, PharmGKB/CPIC, GWAS Catalog, Monarch (phenotypes), AlphaGenome (gRPC).

### Breaking changes from Python BioMCP

- Python package `biomcp-python` is no longer maintained
- MCP tool names and signatures have changed
- Configuration via environment variables only

## 0.7.3

Legacy Python BioMCP. See branch `python-0.7.3` for source code.
