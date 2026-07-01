#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
python_bin="${PYTHON:-python3}"

"$python_bin" - "$repo_root" <<'PY'
import json
import re
import sys
from pathlib import Path

repo = Path(sys.argv[1])
schema_url = "https://static.modelcontextprotocol.io/schemas/2025-12-11/server.schema.json"
expected_name = "io.github.genomoncology/biomcp"


def fail(message: str) -> None:
    print(message, file=sys.stderr)
    sys.exit(1)


def read_text(relative: str) -> str:
    path = repo / relative
    if not path.exists():
        fail(f"missing {relative}")
    return path.read_text(encoding="utf-8")


def first_regex(relative: str, pattern: str) -> str:
    match = re.search(pattern, read_text(relative), re.MULTILINE)
    if not match:
        fail(f"missing version in {relative}")
    return match.group(1)


def json_file(relative: str):
    try:
        return json.loads(read_text(relative))
    except json.JSONDecodeError as exc:
        fail(f"invalid JSON in {relative}: {exc}")


def require_contains(relative: str, needle: str) -> None:
    if needle not in read_text(relative):
        fail(f"missing required text in {relative}: {needle}")


server = json_file("server.json")
if server.get("$schema") != schema_url:
    fail("server.json $schema does not point at the dated MCP Registry schema")
if server.get("name") != expected_name:
    fail("server.json name must be io.github.genomoncology/biomcp")

description = server.get("description")
if not isinstance(description, str) or not description.strip():
    fail("server.json description must be non-empty")
if len(description) > 100:
    fail("server.json description must be at most 100 characters")

packages = server.get("packages")
if not isinstance(packages, list):
    fail("server.json packages must be a list")
package = next(
    (
        entry
        for entry in packages
        if entry.get("registryType") == "pypi"
        and entry.get("identifier") == "biomcp-cli"
    ),
    None,
)
if package is None:
    fail("server.json must include a pypi package entry for biomcp-cli")
if package.get("transport", {}).get("type") != "stdio":
    fail("server.json biomcp-cli package must use stdio transport")
if {"type": "positional", "value": "serve"} not in package.get("packageArguments", []):
    fail("server.json biomcp-cli package must pass positional serve")

versions = {
    "server.json": server.get("version"),
    "server.json packages[biomcp-cli]": package.get("version"),
    "Cargo.toml": first_regex("Cargo.toml", r'^version\s*=\s*"([^"]+)"'),
    "pyproject.toml": first_regex("pyproject.toml", r'^version\s*=\s*"([^"]+)"'),
    "Cargo.lock": first_regex(
        "Cargo.lock", r'name = "biomcp-cli"\nversion = "([^"]+)"'
    ),
    "manifest.json": json_file("manifest.json").get("version"),
    "CITATION.cff": first_regex("CITATION.cff", r'^version:\s*"?([^"\n]+)"?\s*$'),
}
expected_version = versions["Cargo.toml"]
missing = [name for name, value in versions.items() if not value]
if missing:
    fail("missing version in " + ", ".join(missing))
for name, version in versions.items():
    if version != expected_version:
        fail(f"Version mismatch: Cargo.toml={expected_version}, {name}={version}")

readme = read_text("README.md")
require_contains("README.md", "mcp-name: io.github.genomoncology/biomcp")
if "not `biomcp`" not in readme or "unrelated" not in readme:
    fail("README.md must warn that biomcp is an unrelated PyPI package")

pyproject = read_text("pyproject.toml")
if "biomcp-cli" not in pyproject or "biomcp binary" not in pyproject:
    fail("pyproject.toml description must identify biomcp-cli as the PyPI package")

manifest_text = read_text("manifest.json")
if "biomcp-cli" not in manifest_text or "unrelated biomcp" not in manifest_text:
    fail("manifest.json descriptions must disambiguate biomcp-cli from biomcp")

mcp_docs = read_text("docs/reference/mcp-server.md")
for needle in (
    "mcp-publisher init",
    "mcp-publisher login github",
    "mcp-publisher publish",
    "io.github.genomoncology/biomcp",
    "server.json",
):
    if needle not in mcp_docs:
        fail(f"docs/reference/mcp-server.md missing {needle}")
PY

printf 'MCP registry metadata ok\n'
