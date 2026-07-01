#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

extract_version() {
    local file="$1"
    local line
    line="$(grep -m1 -E '^version\s*=\s*"' "$file" || true)"
    if [[ -z "$line" ]]; then
        echo "" && return
    fi
    sed -E 's/^[^"]*"([^"]+)".*$/\1/' <<<"$line"
}

extract_lock_version() {
    local file="$1"
    awk '/name = "biomcp-cli"/{found=1} found && /^version/{print; exit}' "$file" \
        | sed -E 's/^[^"]*"([^"]+)".*$/\1/'
}

extract_manifest_version() {
    local file="$1"
    local line
    line="$(grep -m1 -E '^\s*"version"\s*:\s*"' "$file" || true)"
    if [[ -z "$line" ]]; then
        echo "" && return
    fi
    sed -E 's/^[[:space:]]*"version"[[:space:]]*:[[:space:]]*"([^"]+)".*$/\1/' <<<"$line"
}

extract_citation_version() {
    local file="$1"
    local line
    line="$(grep -m1 -E '^version:[[:space:]]*' "$file" || true)"
    if [[ -z "$line" ]]; then
        echo "" && return
    fi
    sed -E 's/^version:[[:space:]]*"?([^"]+)"?[[:space:]]*$/\1/' <<<"$line"
}

extract_server_version() {
    local file="$1"
    python3 - "$file" <<'PY'
import json
import sys

with open(sys.argv[1], encoding="utf-8") as handle:
    print(json.load(handle).get("version", ""))
PY
}

extract_server_package_version() {
    local file="$1"
    python3 - "$file" <<'PY'
import json
import sys

with open(sys.argv[1], encoding="utf-8") as handle:
    server = json.load(handle)
for package in server.get("packages", []):
    if package.get("registryType") == "pypi" and package.get("identifier") == "biomcp-cli":
        print(package.get("version", ""))
        break
else:
    print("")
PY
}

cargo_version="$(extract_version "$repo_root/Cargo.toml")"
python_version="$(extract_version "$repo_root/pyproject.toml")"
lock_version="$(extract_lock_version "$repo_root/Cargo.lock")"
manifest_version="$(extract_manifest_version "$repo_root/manifest.json")"
citation_version="$(extract_citation_version "$repo_root/CITATION.cff")"
server_version="$(extract_server_version "$repo_root/server.json")"
server_package_version="$(extract_server_package_version "$repo_root/server.json")"

if [[ -z "$cargo_version" || -z "$python_version" || -z "$lock_version" || -z "$manifest_version" || -z "$citation_version" || -z "$server_version" || -z "$server_package_version" ]]; then
    echo "Unable to read version from one or more manifests:" >&2
    echo "  Cargo.toml:              '$cargo_version'" >&2
    echo "  pyproject.toml:          '$python_version'" >&2
    echo "  Cargo.lock:              '$lock_version'" >&2
    echo "  manifest.json:           '$manifest_version'" >&2
    echo "  CITATION.cff:            '$citation_version'" >&2
    echo "  server.json:             '$server_version'" >&2
    echo "  server.json biomcp-cli:  '$server_package_version'" >&2
    exit 1
fi

ok=true

if [[ "$cargo_version" != "$python_version" ]]; then
    echo "Version mismatch: Cargo.toml=$cargo_version, pyproject.toml=$python_version" >&2
    ok=false
fi

if [[ "$cargo_version" != "$lock_version" ]]; then
    echo "Version mismatch: Cargo.toml=$cargo_version, Cargo.lock=$lock_version" >&2
    ok=false
fi

if [[ "$cargo_version" != "$manifest_version" ]]; then
    echo "Version mismatch: Cargo.toml=$cargo_version, manifest.json=$manifest_version" >&2
    ok=false
fi

if [[ "$cargo_version" != "$citation_version" ]]; then
    echo "Version mismatch: Cargo.toml=$cargo_version, CITATION.cff=$citation_version" >&2
    ok=false
fi

if [[ "$cargo_version" != "$server_version" ]]; then
    echo "Version mismatch: Cargo.toml=$cargo_version, server.json=$server_version" >&2
    ok=false
fi

if [[ "$cargo_version" != "$server_package_version" ]]; then
    echo "Version mismatch: Cargo.toml=$cargo_version, server.json biomcp-cli=$server_package_version" >&2
    ok=false
fi

if [[ "$ok" == false ]]; then
    exit 1
fi

echo "Versions in sync: $cargo_version"
