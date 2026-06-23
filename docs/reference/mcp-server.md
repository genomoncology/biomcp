# MCP Server Reference

BioMCP exposes one execution tool (`biomcp`) and a current resource inventory
centered on the help guide. This page documents the stable MCP contract and
executes lightweight checks against the source tree.

## Runtime Surface

BioMCP exposes two MCP entrypoints:

- stdio: `biomcp serve`
- remote Streamable HTTP: `biomcp serve-http`

Manual stdio runs require an MCP client to send the initialize handshake on
stdin. If `biomcp serve` or its `biomcp mcp` alias is launched with stdin closed,
the command exits non-zero and prints recovery guidance that points operators to
`biomcp serve-http` for manual testing.

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

## Tool Description Contract

The runtime `biomcp` description is generated from
`src/cli/list_reference.md`, but the build step emits an MCP-safe read-only
subset. That sanitized description keeps the catalog-only
`study download --list` form, but it must not advertise
`study download <study_id>` or the combined CLI syntax
`study download [--list] [<study_id>]`. CLI-only packaging or mutating
commands such as `skill install`, `ema sync`, `who sync`, `cvx sync`,
`gtr sync`, `who-ivd sync`, `update`, and `uninstall`
must not appear in the MCP tool description. CLI-only cache commands such as
`cache path`, `cache stats`, `cache clean`, and `cache clear` reveal workstation-local paths and filesystem context, so they also stay out of the MCP tool description.

```python
from pathlib import Path

repo_root = Path.cwd()
build = (repo_root / "build.rs").read_text()
tests = (repo_root / "tests/rmcp_client_contract.rs").read_text()

assert "MCP_SHELL_INTRO" in build
assert "read-only biomedical MCP tool" in build
assert "BLOCKED_MCP_DESCRIPTION_TERMS" in build
assert "`skill install`" in build
assert "`ema sync`" in build
assert "`who sync`" in build
assert "`cvx sync`" in build
assert "`gtr sync`" in build
assert "`who-ivd sync`" in build
assert "`update [--check]`" in build
assert "`uninstall`" in build
assert "study download --list" in build
assert "study download [--list] [<study_id>]" in build
assert 'assert!(description.contains("study download --list"))' in tests
assert '"study download [--list] [<study_id>]"' in tests
assert '"who-ivd sync"' in tests
assert '"biomcp cache stats"' in tests
assert '"biomcp cache clean"' in tests
assert '"biomcp cache clear"' in tests
```

## Tool Response Content

The `biomcp` tool keeps non-chart calls text-only. In MCP mode, charted `study`
commands return two success content blocks in order:

- `text` with the normal markdown/table output
- `image` with `mimeType = "image/svg+xml"` and base64-encoded SVG data

MCP chart calls do not write files. If the caller supplies `--output` or `-o`,
the tool returns a tool error instructing the caller to consume the inline image
instead.

Alias fallback is the main exception to the usual CLI stderr contract: failed
`get gene` / `get drug` alias suggestions are returned to MCP as structured JSON
 text content with `_meta.alias_resolution` and `_meta.next_commands` so agents
 can apply their own retry policy without parsing markdown.

Workflow ladders do not add MCP resources. MCP callers that execute BioMCP
commands with `--json` receive the same CLI JSON contract, so first-call
responses can include `_meta.workflow` and `_meta.ladder[]` when a sidecar-backed
ladder trigger matches. `_meta.next_commands` remains the dynamic one-hop
follow-up list; `_meta.ladder[]` is the static multi-step worked example loaded
from the installed `skills/biomcp/use-cases/<slug>.ladder.json` sidecar.

```python
from pathlib import Path

repo_root = Path.cwd()
shell = (repo_root / "src/mcp/shell.rs").read_text()
cli = (repo_root / "src/cli/mod.rs").read_text()

assert "crate::cli::execute_mcp(args)" in shell
assert "CallToolResult::success" in shell
assert 'Content::image(encoded, "image/svg+xml")' in shell
assert "MCP chart responses do not support --output/-o" in cli
assert 'annotations(title = "BioMCP", read_only_hint = true)' in shell
```

## Read-only Allowlist

The MCP `biomcp` tool accepts read-only CLI commands, including `suggest`,
`discover`, `biomcp skill list`, `biomcp skill render`, existing embedded
`biomcp skill <number-or-slug>` lookups, and the exact `study download --list`
catalog lookup.
Mutating or unknown `skill` subcommands remain blocked unless they are added to
the positive MCP skill allowlist. Cache-family commands such as `cache path`,
`cache stats`, `cache clean`, and `cache clear` are also rejected because they reveal workstation-local paths and filesystem context.
In particular, `study download <study_id>` is rejected because installation
performs network and filesystem writes into the local study directory, while
`gtr sync` and `who-ivd sync` are rejected because they refresh local
diagnostic runtime data under local roots. Operators should run study installs
and local-runtime refresh commands directly via the CLI, outside MCP.

```python
from pathlib import Path

repo_root = Path.cwd()
shell = (repo_root / "src/mcp/shell.rs").read_text()
tests = (repo_root / "tests/rmcp_client_contract.rs").read_text()
assert '"suggest" => true' in shell or '| "suggest" => true' in shell
assert '"discover" => true' in shell or '| "discover" => true' in shell
assert '"study" => {' in shell
assert '"download" => args.len() == 4 && args[3] == "--list"' in shell
assert "suggest/discover/skill" in shell or "suggest/discover/skill)." in shell
assert 'matches!(sub.as_str(), "list" | "render")' in shell
assert "show_use_case(&sub).is_ok()" in shell
assert "!matches!(sub.as_str()" not in shell
assert 'biomcp suggest \\"What drugs treat melanoma?\\"' in tests
assert 'biomcp skill list' in tests
assert 'biomcp skill sync' in tests
assert 'biomcp skill install /tmp/biomcp-skills' in tests
assert 'assert!(description.contains("study download --list"))' in tests
assert 'biomcp study download msk_impact_2017' in tests
assert 'biomcp gtr sync' in tests
assert 'biomcp who-ivd sync' in tests
assert 'READ_ONLY_MESSAGE' in tests
assert 'biomcp cache path' in tests
assert 'CACHE_CLI_ONLY_MESSAGE' in tests
assert 'CACHE_FILESYSTEM_MESSAGE' in tests
```

## Resource Catalog

Current builds always publish the help resource and one markdown resource per embedded skill use-case. The help resource is the canonical prompt body, the
same in-memory text returned by `biomcp skill render` over MCP. It does not
contain repo-relative prompt links.

| URI | Name | Notes |
|-----|------|-------|
| `biomcp://help` | BioMCP Overview | Always listed |
| `biomcp://skill/<slug>` | Pattern: ... | Listed when the matching embedded worked example exists |

```python
from pathlib import Path

repo_root = Path.cwd()
shell = (repo_root / "src/mcp/shell.rs").read_text()
use_cases_dir = repo_root / "skills" / "use-cases"
assert "RESOURCE_HELP_URI" in shell
assert 'RawResource::new(RESOURCE_HELP_URI, "BioMCP Overview")' in shell
assert "list_use_case_refs()" in shell
assert use_cases_dir.exists()
assert list(use_cases_dir.glob("*.md"))
```

## Resource Read Mapping

- `biomcp://help` maps to `show_overview()`.
- `biomcp skill render` maps to the same canonical prompt body.
- `biomcp://skill/<slug>` maps to `show_use_case(<slug>)` when an embedded
  worked example exists.
- All successful reads return `text/markdown`.

```python
from pathlib import Path

repo_root = Path.cwd()
shell = (repo_root / "src/mcp/shell.rs").read_text()
outcome = (repo_root / "src/cli/outcome.rs").read_text()
tests = (repo_root / "tests/rmcp_client_contract.rs").read_text()
assert "show_overview()" in shell
assert "render_system_prompt()" in outcome
assert "assert_resource_inventory_and_reads" in tests
assert "assert_read_only_and_policy_calls" in tests
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

Protocol-level checks are implemented in Rust rmcp integration tests plus HTTP surface pytest checks:

- `tests/conftest.py`
- `tests/rmcp_client_contract.rs`
- `tests/test_mcp_http_surface.py`
- `examples/rmcp_streamable_http_contract.rs`

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
assert (repo_root / "tests/rmcp_client_contract.rs").exists()
assert (repo_root / "tests/test_mcp_http_surface.py").exists()
assert (repo_root / "examples/rmcp_streamable_http_contract.rs").exists()
```
