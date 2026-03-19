# MCP Server Reference

BioMCP exposes one execution tool (`biomcp`) and a current resource inventory
centered on the help guide. This page documents the stable MCP contract and
executes lightweight checks against the source tree.

## Runtime Surface

BioMCP exposes two MCP entrypoints:

- stdio: `biomcp serve`
- remote Streamable HTTP: `biomcp serve-http`

The canonical remote endpoint is `/mcp`. Lightweight probe routes are `/health`,
`/readyz`, and `/`.

```python
from pathlib import Path

repo_root = Path.cwd()
shell = (repo_root / "src/mcp/shell.rs").read_text()
assert "StreamableHttpService" in shell
assert '.nest_service("/mcp", service)' in shell
assert '.route("/health", get(health_handler))' in shell
assert '.route("/readyz", get(health_handler))' in shell
assert '.route("/", get(index_handler))' in shell
```

## Capability Advertisement

The server must advertise both tools and resources.

| Capability | Required |
|------------|----------|
| `tools` | enabled |
| `resources` | enabled |

```python
from pathlib import Path

repo_root = Path.cwd()
shell = (repo_root / "src/mcp/shell.rs").read_text()
assert "enable_tools()" in shell
assert "enable_resources()" in shell
```

## Tool Response Content

The `biomcp` tool keeps non-chart calls text-only. In MCP mode, charted `study`
commands return two success content blocks in order:

- `text` with the normal markdown/table output
- `image` with `mimeType = "image/svg+xml"` and base64-encoded SVG data

MCP chart calls do not write files. If the caller supplies `--output` or `-o`,
the tool returns a tool error instructing the caller to consume the inline image
instead.

```python
from pathlib import Path

repo_root = Path.cwd()
shell = (repo_root / "src/mcp/shell.rs").read_text()
cli = (repo_root / "src/cli/mod.rs").read_text()

assert "crate::cli::execute_mcp(args)" in shell
assert "CallToolResult::success" in shell
assert 'Content::image(encoded, "image/svg+xml")' in shell
assert "MCP chart responses do not support --output/-o" in cli
```

## Resource Catalog

Current builds always publish the help resource:

| URI | Name | Notes |
|-----|------|-------|
| `biomcp://help` | BioMCP Overview | Always listed |

No `biomcp://skill/<slug>` resources are currently listed because the embedded
`skills/` tree ships no `use-cases/*.md` files.

```python
from pathlib import Path

repo_root = Path.cwd()
shell = (repo_root / "src/mcp/shell.rs").read_text()
use_cases_dir = repo_root / "skills" / "use-cases"
assert "RESOURCE_HELP_URI" in shell
assert 'RawResource::new(RESOURCE_HELP_URI, "BioMCP Overview")' in shell
assert "list_use_case_refs()" in shell
assert not use_cases_dir.exists() or list(use_cases_dir.glob("*.md")) == []
```

## Resource Read Mapping

- `biomcp://help` maps to `show_overview()`.
- Compatibility reads for `biomcp://skill/<slug>` map to `show_use_case(<slug>)`
  when an embedded use-case exists.
- All successful reads return `text/markdown`.

```python
from pathlib import Path

repo_root = Path.cwd()
shell = (repo_root / "src/mcp/shell.rs").read_text()
assert "show_overview()" in shell
assert 'if let Some(slug) = uri.strip_prefix("biomcp://skill/")' in shell
assert "show_use_case(slug)" in shell
assert 'with_mime_type("text/markdown")' in shell
```

## Unknown URI Behavior

Unknown resource URIs must return an MCP resource-not-found error and include a helpful message.

```python
from pathlib import Path

repo_root = Path.cwd()
shell = (repo_root / "src/mcp/shell.rs").read_text()
assert "resource_not_found" in shell
assert "Unknown resource:" in shell
```

## Companion Runtime Tests

Protocol-level checks are implemented in Python integration tests:

- `tests/conftest.py`
- `tests/test_mcp_contract.py`
- `tests/test_mcp_http_surface.py`
- `tests/test_mcp_http_transport.py`

These tests validate both transport modes:

- `biomcp serve` stdio initialize/resource behavior,
- stdio charted-study `text` + `image/svg+xml` responses and MCP `--output` rejection,
- Streamable HTTP `initialize`/`tools/list`/`tools/call`,
- Streamable HTTP charted-study `text` + `image/svg+xml` responses,
- `GET /`, `GET /health`, and `GET /readyz`,
- invalid URI error semantics.

```python
from pathlib import Path

repo_root = Path.cwd()
assert (repo_root / "tests/conftest.py").exists()
assert (repo_root / "tests/test_mcp_contract.py").exists()
assert (repo_root / "tests/test_mcp_http_surface.py").exists()
assert (repo_root / "tests/test_mcp_http_transport.py").exists()
```
