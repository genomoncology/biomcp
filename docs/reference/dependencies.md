# Dependencies

BioMCP is a single Rust binary. All dependencies are compiled in — no runtime installations required.

## Key Dependencies

### Kuva — Charting

[Kuva](https://github.com/Psy-Fer/kuva) (v0.1.4) is the charting engine behind BioMCP's `--chart` flag. It renders 8 chart types (bar, pie, histogram, density, box, violin, ridgeline, survival) to terminal, SVG, and PNG.

Kuva is linked as a Rust library, not called as a subprocess. Charts are generated in-process with no additional runtime dependencies.

See the [charting guide](../blog/kuva-charting-guide.md) for examples of every chart type.

### RMCP — Model Context Protocol

[RMCP](https://github.com/anthropics/rust-mcp-sdk) (v1.1.1) provides the MCP server implementation. BioMCP supports both stdio transport (for local use with Claude Desktop) and streamable HTTP transport (for remote deployment).

### Reqwest — HTTP Client

[Reqwest](https://github.com/seanmonstar/reqwest) (v0.12) handles all upstream API calls to biomedical databases. BioMCP adds middleware for:

- **Retries** via `reqwest-retry` — automatic retry with exponential backoff
- **Caching** via `http-cache-reqwest` — local HTTP cache to avoid redundant API calls
- **TLS** via `rustls` — no OpenSSL dependency

### Clap — CLI Framework

[Clap](https://github.com/clap-rs/clap) (v4) generates the command-line interface. BioMCP's `search`, `get`, `study`, and `chart` command hierarchy is defined using Clap's derive macros.

### Tokio — Async Runtime

[Tokio](https://github.com/tokio-rs/tokio) (v1) provides the async runtime. BioMCP uses parallel API fan-out — multiple upstream sources are queried concurrently for faster responses.

### Tonic + Prost — gRPC

[Tonic](https://github.com/hyperium/tonic) (v0.12) and [Prost](https://github.com/tokio-rs/prost) (v0.13) support gRPC communication with services that use Protocol Buffers.

### MiniJinja — Templating

[MiniJinja](https://github.com/mitsuhiko/minijinja) (v2) renders BioMCP's markdown output templates. Each entity type's output format is defined as a Jinja template.

## Data Processing

| Crate | Version | Purpose |
|-------|---------|---------|
| `serde` / `serde_json` | 1.0 | JSON serialization for all API responses |
| `csv` | 1.0 | cBioPortal study data parsing (TSV files) |
| `roxmltree` | 0.20 | XML parsing for PubMed and ClinicalTrials.gov |
| `regex` | 1.0 | Pattern matching for variant notation parsing |
| `flate2` | 1.0 | Gzip decompression for downloaded study archives |
| `tar` | 0.4 | Tar archive extraction for study downloads |
| `zip` | 0.6 | ZIP archive handling |
| `zstd` | 0.13 | Zstandard compression |

## Security

| Crate | Version | Purpose |
|-------|---------|---------|
| `sha2` | 0.10 | SHA-256 hashing for cache keys |
| `md5` | 0.7 | MD5 hashing for content deduplication |
| `base64` | 0.22 | Base64 encoding for binary data |

## Full Dependency List

BioMCP's complete dependency tree is defined in [`Cargo.toml`](https://github.com/genomoncology/biomcp/blob/main/Cargo.toml) and locked in `Cargo.lock`.
