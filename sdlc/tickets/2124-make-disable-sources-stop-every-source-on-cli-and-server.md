# 2124 — Make disable-sources stop every source on the CLI and the server

Status: OPEN.

Milestone: 0.9.3

## Outcome

Any source id `--disable-sources` or `BIOMCP_DISABLE_SOURCES` accepts is never called by any command, on the CLI or the MCP server, and the flag works the same on `biomcp serve` and `serve-http` as the environment variable does.

## Evidence

Found by the 0.9.3 adversarial review on 2026-10-16, against 2061 (landed 35f42b858).

- `is_disabled(` is checked only for kegg, enrichr, oncokb, ddinter, drugbank, pharmacodb and the cosmic and cancer-genome-interpreter fields. The parser accepts every id in `sources.json`, and `run_api_probes` (`src/cli/health/runner.rs:283-320`) marks any disabled id "Disabled" and skips its probe. So `--disable-sources civic` reports CIViC off in health and the MCP description while `get variant` still calls CIViC.
- `--disable-sources` is global (`src/cli/types.rs:29`), but `main.rs:136-150` sends Serve and ServeHttp to `mcp::run_stdio`/`run_http`, which call `disable::init(None)` (`src/mcp/mod.rs:17,39`); the flag is ignored there.
- CHANGELOG.md line 7 and `docs/reference/configuration.md:166` promise a disabled source is never called. Users disable sources for licensing reasons.

- Starts from: `src/sources/disable.rs`, the shared HTTP client the source clients build on, `src/main.rs`, `src/mcp/mod.rs`.
- Decisions:
  - Enforce the check once, where every source client sends its request, keyed by the source's registry id, so a new source cannot skip it. A disabled source answers like an unavailable section with the reason "disabled by configuration"; it never reaches the network.
  - The serve commands pass the parsed flag into `disable::init`; flag and environment variable combine as on the CLI.
- Changes: claims `src/sources/disable.rs` `src/sources/**` (client construction only) `src/main.rs` `src/mcp/mod.rs` `src/cli/health/**` `docs/reference/configuration.md` `CHANGELOG.md` and tests beside them.
- Proof, each failing on current main: a test walks every id in `sources.json`, disables it, runs a command that uses it against the replay server, and asserts no request for that source reaches the server; `biomcp --disable-sources civic serve` lists CIViC as off and a `get variant` tool call sends no CIViC request.
- Order: before release; shares no file with the open speed tickets.
