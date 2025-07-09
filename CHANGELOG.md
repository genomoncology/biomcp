# Changelog

All notable changes to the BioMCP project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Unified search and fetch tools following OpenAI MCP guidelines
- Additional variant sources (TCGA/GDC, 1000 Genomes) enabled by default in fetch operations
- Additional article sources (bioRxiv, medRxiv, Europe PMC) enabled by default in search operations
- **cBioPortal Integration** for article searches:
  - Automatic gene-level mutation summaries when searching with gene parameters
  - Mutation-specific search capabilities (e.g., BRAF V600E, SRSF2 F57\*)
  - Dynamic cancer type resolution using cBioPortal API
  - Smart caching and rate limiting for optimal performance
- Comprehensive constants module for better maintainability
- Domain-specific handlers for result formatting
- Parameter parser for robust input validation
- Custom exception hierarchy for better error handling
- Support for legacy trial result format from ClinicalTrials.gov API
- Rate limiting with token bucket and sliding window algorithms
- Retry logic with exponential backoff for transient failures
- Performance monitoring and metrics collection system
- GitHub Actions CI/CD workflows for automated testing and releases
- Session-based thinking to replace global state
- Extracted router handlers to reduce complexity
- **Performance Optimizations**:
  - Connection pooling with event loop lifecycle management (30% latency reduction)
  - Parallel test execution with pytest-xdist (5x faster test runs)
  - Request batching for cBioPortal API calls (80% fewer API calls)
  - Smart caching with LRU eviction and fast hash keys (10x faster cache operations)
  - Prefetching for common genes, diseases, and chemicals
  - Shared validation context to eliminate redundant checks
  - Pagination support for Europe PMC searches
  - Conditional metrics collection (disabled by default)
- Integration tests for MCP server functionality
- Utility modules for gene validation, mutation filtering, and request caching

### Changed

- Consolidated 10 separate MCP tools into 2 unified tools (search and fetch)
- Improved error handling with specific exception types
- Enhanced type safety throughout the codebase
- Refactored domain handlers to use static methods for better performance
- Updated response formats to comply with OpenAI MCP specifications
- Replaced global state in thinking module with session management
- Refactored complex functions to meet code quality standards
- Added domain parameter to all API calls for proper retry configuration

### Removed

- Individual domain-specific MCP tool decorators
- Duplicate handler implementations in src/biomcp/handlers/
- Global state from sequential thinking module

### Fixed

- Empty trial results when ClinicalTrials.gov returns legacy format
- Variant fetch returning list instead of single object
- Type errors in router.py for full mypy compliance
- Trial fetch error when API returns wrapped response structure
- Complex functions exceeding cyclomatic complexity thresholds
- Race conditions in sequential thinking with concurrent usage

## [0.1.10] - 2024-01-15

### Added

- Initial release of BioMCP
- PubMed/PubTator3 article search integration
- ClinicalTrials.gov trial search integration
- MyVariant.info variant search integration
- Sequential thinking tool for systematic problem-solving
- CLI interface for direct usage
- MCP server for AI assistant integration
- Cloudflare Worker support for remote deployment
- Comprehensive test suite with pytest-bdd

### Security

- API keys properly externalized
- Input validation using Pydantic models
- Safe string handling in all API calls

[Unreleased]: https://github.com/genomoncology/biomcp/compare/v0.1.10...HEAD
[0.1.10]: https://github.com/genomoncology/biomcp/releases/tag/v0.1.10
