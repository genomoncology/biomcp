# Changelog

## [0.4.0] - 2025-06-19

### Added

- **Unified Query Language**: Integrated into the `search` tool
  - Use `query` parameter for structured field syntax
  - Natural language queries: `"BRAF melanoma"`
  - Field-specific queries: `"gene:BRAF AND trials.condition:melanoma"`
  - Cross-domain searches from a single query
  - Support for boolean operators (AND, OR, NOT) and comparisons (>, <, ..)
- **Schema Discovery**: Access via `search(get_schema=True)`
- **Query Explanation**: Use `explain_query=True` to see how queries are parsed
- **Integrated Sequential Thinking**: Access via `search(domain="thinking")`
  - Clients should explicitly call this before biomedical searches
  - Provides systematic step-by-step analysis
  - Supports thought revision and branching

### Changed

- **Two Core Tools**: `search` and `fetch` (following OpenAI MCP guidelines)
- **Sequential Thinking Integration**: Now accessible through `search(domain="thinking")` instead of a separate tool
- **Triple-Mode Search**: The `search` tool supports sequential thinking, unified query, and legacy domain-based modes
- **Backward Compatible**: All existing integrations continue to work

### Enhanced

- Improved result formatting for all search domains
- Better handling of JSON string parameters from MCP clients
- More robust error handling and result parsing

### Fixed

- Fixed variant fetch error with incorrect parameter names (`include_external` instead of `include_tcga`)
- Fixed article fetch error where `_article_details` was called with non-existent `fetch_full_text` parameter
- Fixed article search returning 0 results due to overly restrictive AND logic for keywords
- Fixed JSON parsing error in article fetch by calling `fetch_articles` directly with `output_json=True`
- Fixed trial search "'list' object has no attribute 'get'" error by handling ClinicalTrials.gov API v2 response format
- Fixed trial search phase validation error by normalizing phase values (e.g., "Phase 3" → "PHASE3")
- Fixed trial search KeyError 'nct_id' by handling nested ClinicalTrials.gov API v2 structure in `format_results`
- Fixed parameter validation errors by accepting both string and list types for search parameters

## [0.3.0] - 2025-06-19

### Breaking Changes

- **MCP Tools Consolidation**: Consolidated 10 separate MCP tools into 2 unified tools (`search` and `fetch`) to align with OpenAI MCP guidelines
  - Removed individual MCP tool decorators from legacy functions
  - Legacy functions renamed with leading underscore to indicate internal use
  - Sequential thinking tool removed from MCP interface (now internal)

### Added

- **Unified Search Tool**: Single `search` tool that handles articles, trials, and variants with domain parameter
- **Unified Fetch Tool**: Single `fetch` tool that retrieves details for any domain type
- **OpenAI MCP Compatibility**: Search results now follow OpenAI's standardized format with id, title, snippet, url, and metadata fields

### Changed

- CLI interface remains unchanged - all `biomcp` commands continue to work as before
- Internal functions preserved for backward compatibility with CLI

### Migration Guide

If you were using the MCP tools directly:

- Replace `article_searcher(...)` with `search(domain="article", ...)`
- Replace `trial_searcher(...)` with `search(domain="trial", ...)`
- Replace `variant_searcher(...)` with `search(domain="variant", ...)`
- Replace `article_details(pmid=X)` with `fetch(domain="article", id=X)`
- Replace `trial_protocol(nct_id=X)` with `fetch(domain="trial", id=X, detail="protocol")`
- Remove usage of `sequential_thinking` tool (use LLM's internal reasoning instead)

## [0.1.1] - 2025-04-14

- Simplified `biomcp run`.
- Added tutorials on Claude Desktop, MCP Inspector, and Python SDK.

## [0.1.0] - 2025-04-08

- Initial release of BioMCP CLI and server.
- Support for searching ClinicalTrials.gov (`biomcp trial search`).
- Support for retrieving trial details (`biomcp trial get`).
- Support for searching PubMed/PubTator3 (`biomcp article search`).
- Support for retrieving article details (`biomcp article get`).
- Support for searching MyVariant.info (`biomcp variant search`).
- Support for retrieving variant details (`biomcp variant get`).
- Basic HTTP caching for API requests.
- Initial documentation structure.
