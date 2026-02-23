# MCP Server Reference

BioMCP exposes one execution tool (`shell`) and a curated resource catalog for agent guidance.
This page documents the stable MCP contract and executes lightweight checks against the source tree.

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

## Resource Catalog

BioMCP publishes a help resource plus one markdown resource per registered skill use-case.

| URI | Name |
|-----|------|
| `biomcp://help` | BioMCP Overview |
| `biomcp://skill/<slug>` | Pattern: `<title>` |

```python
import re
from pathlib import Path

repo_root = Path.cwd()
shell = (repo_root / "src/mcp/shell.rs").read_text()
assert "RESOURCE_HELP_URI" in shell
assert 'uri: RESOURCE_HELP_URI.to_string()' in shell
assert 'name: "BioMCP Overview".to_string()' in shell
assert "list_use_case_refs()" in shell
assert 'format!("biomcp://skill/{}", skill.slug)' in shell
assert 'name = if title.to_ascii_lowercase().starts_with("pattern:")' in shell
assert len(set(re.findall(r"biomcp://[a-z0-9/-]+", shell))) >= 2
```

## Resource Read Mapping

- `biomcp://help` maps to `show_overview()`.
- Skill URIs map to `show_use_case(<slug>)`.
- All successful reads return `text/markdown`.

```python
from pathlib import Path

repo_root = Path.cwd()
shell = (repo_root / "src/mcp/shell.rs").read_text()
assert "show_overview()" in shell
assert 'if let Some(slug) = uri.strip_prefix("biomcp://skill/")' in shell
assert "show_use_case(slug)" in shell
assert 'mime_type: Some("text/markdown".to_string())' in shell
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

These tests run a real `biomcp serve` session over stdio and validate:

- initialize handshake,
- tools inventory,
- resource inventory,
- resource reads,
- invalid URI error semantics.

```python
from pathlib import Path

repo_root = Path.cwd()
assert (repo_root / "tests/conftest.py").exists()
assert (repo_root / "tests/test_mcp_contract.py").exists()
```
