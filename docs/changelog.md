# Changelog

All notable changes to the BioMCP project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.6.2] - 2025-08-05

### Added

- **NCI Clinical Trials Search API Integration** - Enhanced cancer trial search capabilities:
  - Dual source support for trial search/getter tools (ClinicalTrials.gov + NCI)
  - NCI API key handling via `NCI_API_KEY` environment variable or parameter
  - Advanced trial filters: biomarkers, prior therapy, brain metastases acceptance
  - **6 New MCP Tools** for NCI-specific searches:
    - `nci_organization_searcher` / `nci_organization_getter`: Cancer centers, hospitals, research institutions
    - `nci_intervention_searcher` / `nci_intervention_getter`: Drugs, devices, procedures, biologicals
    - `nci_biomarker_searcher`: Trial eligibility biomarkers (reference genes, branches)
    - `nci_disease_searcher`: NCI's controlled vocabulary of cancer conditions
  - **OR Query Support**: All NCI endpoints support OR queries (e.g., "PD-L1 OR CD274")
  - Real-time access to NCI's curated cancer trials database
  - Automatic cBioPortal integration for gene searches
  - Proper NCI parameter mapping (org_city, org_state_or_province, etc.)
  - Comprehensive error handling for Elasticsearch limits

### Changed

- Enhanced unified search router to properly handle NCI domains
- Trial search/getter tools now accept `source` parameter ("clinicaltrials" or "nci")
- Improved domain-specific search logic for query+domain combinations

### Added CLI Commands

```bash
# Organization search/get
biomcp organization search "MD Anderson" --api-key YOUR_KEY
biomcp organization get 12345 --api-key YOUR_KEY

# Intervention search/get
biomcp intervention search pembrolizumab --type Drug --api-key YOUR_KEY
biomcp intervention get 67890 --api-key YOUR_KEY

# Biomarker search
biomcp biomarker search --name "PD-L1" --api-key YOUR_KEY

# Disease search
biomcp disease search melanoma --source nci --api-key YOUR_KEY

# Enhanced trial commands with source selection
biomcp trial search --condition melanoma --source nci --api-key YOUR_KEY
biomcp trial get NCT04280705 --source nci --api-key YOUR_KEY
```

### Documentation

- Added NCI tutorial with example prompts: `docs/tutorials/nci-prompts.md`
- Created API parameter reference: `docs/api-changes/nci-api-parameters.md`
- Updated CLAUDE.md with NCI usage instructions and parameter notes
- Requires NCI API key from: https://clinicaltrialsapi.cancer.gov/

## [0.6.0] - 2025-08-01

### Added

- **Streamable HTTP Transport Support** (#45) - MCP specification version 2025-03-26:
  - Enabled FastMCP's native `/mcp` endpoint for Streamable HTTP transport
  - MCP specification compliant transport (2025-03-26 spec) via FastMCP 1.12.3+
  - CLI support via `biomcp run --mode streamable_http` (uses native FastMCP implementation)
  - Full backward compatibility with legacy SSE endpoints
  - Cloudflare Worker updated with POST /mcp route for full spec compliance
  - Simplified worker implementation to leverage FastMCP's built-in transport support
  - Added comprehensive integration tests for streamable HTTP functionality
  - New transport protocol documentation guide

### Changed

- Enhanced CLI with transport modes (stdio, worker, streamable_http)
- Added configurable host and port options for HTTP-based transports
- Simplified server modes by removing redundant `http` mode
- Cloudflare Worker now supports both GET and POST methods on /mcp endpoint
- Pinned FastMCP dependency to version range >=1.12.3,<2.0.0 for stability
- Standardized documentation file naming to lowercase with hyphens for consistency

### Migration Notes

- **From SSE to Streamable HTTP**: Update your server startup from `--mode worker` to `--mode streamable_http`
- **Docker deployments**: Ensure you're using `--host 0.0.0.0` for proper container networking
- **Cloudflare Workers**: The worker now automatically handles both transport types on `/mcp`
- See the new [Transport Protocol Guide](https://biomcp.org/transport-protocol/) for detailed migration instructions

## [0.5.0] - 2025-08-01

### Added

- **BioThings Integration** for real-time biomedical data access:
  - **New MCP Tools** (3 tools added, total now 17):
    - `gene_getter`: Query MyGene.info for gene information (symbols, names, summaries)
    - `drug_getter`: Query MyChem.info for drug/chemical data (formulas, indications, mechanisms)
    - `disease_getter`: Query MyDisease.info for disease information (definitions, synonyms, ontologies)
  - **Unified Search/Fetch Enhancement**:
    - Added `gene`, `drug`, `disease` as new searchable domains alongside article, trial, variant
    - Integrated into unified search syntax: `search(domain="gene", keywords=["BRAF"])`
    - Query language support: `gene:BRAF`, `drug:pembrolizumab`, `disease:melanoma`
    - Full fetch support: `fetch(domain="drug", id="DB00945")`
  - **Clinical Trial Enhancement**:
    - Automatic disease synonym expansion for trial searches
    - Real-time synonym lookup from MyDisease.info
    - Example: searching for "GIST" automatically includes "gastrointestinal stromal tumor"
  - **Smart Caching & Performance**:
    - Batch operations for multiple gene/drug lookups
    - Intelligent caching with TTL (gene: 24h, drug: 48h, disease: 72h)
    - Rate limiting to respect API guidelines

### Changed

- Trial search now expands disease terms by default (disable with `expand_synonyms=False`)
- Enhanced error handling for BioThings API responses
- Improved network reliability with automatic retries

## [0.4.6] - 2025-07-09

### Added

- MkDocs documentation deployment

## [0.4.5] - 2025-07-09

### Added

- Unified search and fetch tools following OpenAI MCP guidelines
- Additional variant sources (TCGA/GDC, 1000 Genomes) enabled by default in fetch operations
- Additional article sources (bioRxiv, medRxiv, Europe PMC) enabled by default in search operations

### Changed

- Consolidated 10 separate MCP tools into 2 unified tools (search and fetch)
- Updated response formats to comply with OpenAI MCP specifications

### Fixed

- OpenAI MCP compliance issues to enable integration

## [0.4.4] - 2025-07-08

### Added

- **Performance Optimizations**:
  - Connection pooling with event loop lifecycle management (30% latency reduction)
  - Parallel test execution with pytest-xdist (5x faster test runs)
  - Request batching for cBioPortal API calls (80% fewer API calls)
  - Smart caching with LRU eviction and fast hash keys (10x faster cache operations)
  - Major performance improvements achieving ~3x faster test execution (120s → 42s)

### Fixed

- Non-critical ASGI errors suppressed
- Performance issues in article_searcher

## [0.4.3] - 2025-07-08

### Added

- Complete HTTP centralization and improved code quality
- Comprehensive constants module for better maintainability
- Domain-specific handlers for result formatting
- Parameter parser for robust input validation
- Custom exception hierarchy for better error handling

### Changed

- Refactored domain handlers to use static methods for better performance
- Enhanced type safety throughout the codebase
- Refactored complex functions to meet code quality standards

### Fixed

- Type errors in router.py for full mypy compliance
- Complex functions exceeding cyclomatic complexity thresholds

## [0.4.2] - 2025-07-07

### Added

- Europe PMC DOI support for article fetching
- Pagination support for Europe PMC searches
- OR logic support for variant notation searches (e.g., R173 vs Arg173 vs p.R173)

### Changed

- Enhanced variant notation search capabilities

## [0.4.1] - 2025-07-03

### Added

- AlphaGenome as an optional dependency to predict variant effects on gene regulation
- Per-request API key support for AlphaGenome integration
- AI predictions to complement existing database lookups

### Security

- Comprehensive sanitization in Cloudflare Worker to prevent sensitive data logging
- Secure usage in hosted environments where users provide their own keys

## [0.4.0] - 2025-06-27

### Added

- **cBioPortal Integration** for article searches:
  - Automatic gene-level mutation summaries when searching with gene parameters
  - Mutation-specific search capabilities (e.g., BRAF V600E, SRSF2 F57\*)
  - Dynamic cancer type resolution using cBioPortal API
  - Smart caching and rate limiting for optimal performance

## [0.3.3] - 2025-06-20

### Changed

- Release workflow updates

## [0.3.2] - 2025-06-20

### Changed

- Release workflow updates

## [0.3.1] - 2025-06-20

### Fixed

- Build and release process improvements

## [0.3.0] - 2025-06-20

### Added

- Expanded search capabilities
- Integration tests for MCP server functionality
- Utility modules for gene validation, mutation filtering, and request caching

## [0.2.1] - 2025-06-19

### Added

- Remote MCP policies

## [0.2.0] - 2025-06-17

### Added

- Sequential thinking tool for systematic problem-solving
- Session-based thinking to replace global state
- Extracted router handlers to reduce complexity

### Changed

- Replaced global state in thinking module with session management

### Removed

- Global state from sequential thinking module

### Fixed

- Race conditions in sequential thinking with concurrent usage

## [0.1.11] - 2025-06-12

### Added

- Advanced eligibility criteria filters to clinical trial search

## [0.1.10] - 2025-05-21

### Added

- OAuth support on the Cloudflare worker via Stytch

## [0.1.9] - 2025-05-17

### Fixed

- Refactor: Bump minimum Python version to 3.10

## [0.1.8] - 2025-05-14

### Fixed

- Article searcher fixes

## [0.1.7] - 2025-05-07

### Added

- Remote OAuth support

## [0.1.6] - 2025-05-05

### Added

- Updates to handle cursor integration

## [0.1.5] - 2025-05-01

### Added

- Updates to smithery yaml to account for object types needed for remote calls
- Documentation and Lzyank updates

## [0.1.3] - 2025-05-01

### Added

- Health check functionality to assist with API call issues
- System resources and network & environment information gathering
- Remote MCP capability via Cloudflare using SSE

## [0.1.2] - 2025-04-18

### Added

- Researcher persona and BioMCP v0.1.2 release
- Deep Researcher Persona blog post
- Researcher persona video demo

## [0.1.1] - 2025-04-14

### Added

- Claude Desktop and MCP Inspector tutorials
- Improved Claude Desktop Tutorial for BioMCP
- Troubleshooting guide and blog post

### Fixed

- Log tool names as comma separated string
- Server hanging issues
- Error responses in variant count check

## [0.1.0] - 2025-04-08

### Added

- Initial release of BioMCP
- PubMed/PubTator3 article search integration
- ClinicalTrials.gov trial search integration
- MyVariant.info variant search integration
- CLI interface for direct usage
- MCP server for AI assistant integration
- Cloudflare Worker support for remote deployment
- Comprehensive test suite with pytest-bdd
- GenomOncology introduction
- Blog post on AI-assisted clinical trial search
- MacOS troubleshooting guide

### Security

- API keys properly externalized
- Input validation using Pydantic models
- Safe string handling in all API calls

[Unreleased]: https://github.com/genomoncology/biomcp/compare/v0.6.2...HEAD
[0.6.2]: https://github.com/genomoncology/biomcp/releases/tag/v0.6.2
[0.6.0]: https://github.com/genomoncology/biomcp/releases/tag/v0.6.0
[0.5.0]: https://github.com/genomoncology/biomcp/releases/tag/v0.5.0
[0.4.6]: https://github.com/genomoncology/biomcp/releases/tag/v0.4.6
[0.4.5]: https://github.com/genomoncology/biomcp/releases/tag/v0.4.5
[0.4.4]: https://github.com/genomoncology/biomcp/releases/tag/v0.4.4
[0.4.3]: https://github.com/genomoncology/biomcp/releases/tag/v0.4.3
[0.4.2]: https://github.com/genomoncology/biomcp/releases/tag/v0.4.2
[0.4.1]: https://github.com/genomoncology/biomcp/releases/tag/v0.4.1
[0.4.0]: https://github.com/genomoncology/biomcp/releases/tag/v0.4.0
[0.3.3]: https://github.com/genomoncology/biomcp/releases/tag/v0.3.3
[0.3.2]: https://github.com/genomoncology/biomcp/releases/tag/v0.3.2
[0.3.1]: https://github.com/genomoncology/biomcp/releases/tag/v0.3.1
[0.3.0]: https://github.com/genomoncology/biomcp/releases/tag/v0.3.0
[0.2.1]: https://github.com/genomoncology/biomcp/releases/tag/v0.2.1
[0.2.0]: https://github.com/genomoncology/biomcp/releases/tag/v0.2.0
[0.1.11]: https://github.com/genomoncology/biomcp/releases/tag/v0.1.11
[0.1.10]: https://github.com/genomoncology/biomcp/releases/tag/v0.1.10
[0.1.9]: https://github.com/genomoncology/biomcp/releases/tag/v0.1.9
[0.1.8]: https://github.com/genomoncology/biomcp/releases/tag/v0.1.8
[0.1.7]: https://github.com/genomoncology/biomcp/releases/tag/v0.1.7
[0.1.6]: https://github.com/genomoncology/biomcp/releases/tag/v0.1.6
[0.1.5]: https://github.com/genomoncology/biomcp/releases/tag/v0.1.5
[0.1.3]: https://github.com/genomoncology/biomcp/releases/tag/v0.1.3
[0.1.2]: https://github.com/genomoncology/biomcp/releases/tag/v0.1.2
[0.1.1]: https://github.com/genomoncology/biomcp/releases/tag/v0.1.1
[0.1.0]: https://github.com/genomoncology/biomcp/releases/tag/v0.1.0
