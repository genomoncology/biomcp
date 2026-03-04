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

cargo_version="$(extract_version "$repo_root/Cargo.toml")"
python_version="$(extract_version "$repo_root/pyproject.toml")"

if [[ -z "$cargo_version" || -z "$python_version" ]]; then
    echo "Unable to read version from Cargo.toml or pyproject.toml" >&2
    exit 1
fi

if [[ "$cargo_version" != "$python_version" ]]; then
    echo "Version mismatch: Cargo.toml=$cargo_version, pyproject.toml=$python_version" >&2
    exit 1
fi

echo "Versions in sync: $cargo_version"
