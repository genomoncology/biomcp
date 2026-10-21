# Features

Each stable name describes one caller-visible behavior. Tests declare their owning feature in `tests/feature-links.json`; golden cases derive it from their existing command arguments. Internal tests do not contribute outside-in coverage.

- gene: Find genes and retrieve their source-backed details and relationships.
- variant: Resolve variants and retrieve their identity, annotations and matching literature.
- drug: Find drugs and retrieve their identity, targets, regulatory records and interactions.
- article: Find articles and retrieve faithful metadata, annotations, full text and assets.
- author: Find provider-identified authors and retrieve their profiles and papers.
- disease: Find diseases and retrieve their source-backed associations.
- protein: Find proteins and retrieve structures and relationships.
- pathway: Find pathways and retrieve their genes and events.
- cell-line: Find cell lines and retrieve their identity, variants and cross-references.
- pgx: Find pharmacogenomic records and retrieve guidelines, recommendations and interactions.
- gwas: Search reported genetic associations with provenance and bounded results.
- phenotype: Search phenotypes and their recorded associations.
- diagnostic: Retrieve diagnostic test details from the configured source.
- search-all: Search across entity types while preserving each source outcome.
- cache: Inspect and manage cached responses while preserving caller data and process safety.
- no-cache: Bypass response caching without reading or changing managed cache state.
- source-policy: Honor configured source exclusions and report provider outcomes truthfully.
- mcp-stdio: Serve commands over standard-input MCP with protocol validation and recovery.
- mcp-http: Serve commands over Streamable HTTP with route and session boundaries.
- mcp-catalog: Expose the public MCP tool catalog and its argument schemas.
- mcp-parity: Preserve command results and errors across the CLI and MCP transports.
- study: Query local study data and display cohort summaries, comparisons and charts.
- managed-data: Use configured local data bundles and report their availability.
- health: Report provider and local-data health without exposing local paths.
- cli-reference: Describe available commands, arguments and their public output formats.
- skills: Show and install the packaged command guidance and workflows.
- install-update: Install, update and remove only the owned executable and installation receipt.
- adverse-event: Search adverse-event reports with bounded filters and source context.
- output-safety: Preserve structured output, binary assets and terminal safety at caller boundaries.
- operator-tls: Honor operator certificate bundles without weakening transport security.
- provider-retry: Handle provider rate limits and retries within the published policy.
- trial: Find clinical trials and retrieve verified eligibility, sites, interventions and documents.
- discovery: Resolve free-text terms to a supported entity and useful follow-up commands.
- cli-controls: Apply global command options consistently before provider or filesystem work.

`make lint` checks these links without building instrumented code. `make feature-coverage` runs the offline CLI/MCP consumers with LLVM instrumentation and compares unique executable product source regions with `tool.biomcp.feature-coverage.floor` in `pyproject.toml`. Nightly checks and both release qualification entry points call that target. The collector prepares the pinned coverage executable, matching Rust LLVM tools and the existing spec runner; Cargo nextest must be available. Coverage collection requires a Linux host.

Ticket 2342 records the source-region baseline reset. The denominator excludes unit suites, test-only source, generated source, inline test modules and compiler-omitted code. The collection does not execute opt-in live or ignored cases, other-platform cases or spec pages outside the existing offline selection. Each completed collection writes its exact source, suite selections, unreached regions and feature/case associations under `.cache/feature-coverage/coverage.json`. A region remains unreached when its count is zero across all collected binaries, including a closure on an otherwise executed line. A named case documents the associated behavior; it does not prove that the missing regions were reached. Files absent from the executable mapping remain explicit, including declaration/module glue. Failed or partial suites produce no percentage.
