# Changelog

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
