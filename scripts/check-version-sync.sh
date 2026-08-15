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
    python3 - "$1" <<'PY'
import json
import sys
with open(sys.argv[1], encoding="utf-8") as handle:
    print(json.load(handle).get("version", ""))
PY
}

extract_server_package_version() {
    python3 - "$1" <<'PY'
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

extract_formula_version() {
    local line
    line="$(grep -m1 -E '^  version "' "$1" || true)"
    if [[ -z "$line" ]]; then
        echo "" && return
    fi
    sed -E 's/^  version "([^"]+)".*$/\1/' <<<"$line"
}

cargo_version="$(extract_version "$repo_root/Cargo.toml")"
python_version="$(extract_version "$repo_root/pyproject.toml")"
lock_version="$(extract_lock_version "$repo_root/Cargo.lock")"
uv_lock_version="$(extract_lock_version "$repo_root/uv.lock")"
manifest_version="$(extract_manifest_version "$repo_root/manifest.json")"
citation_version="$(extract_citation_version "$repo_root/CITATION.cff")"
server_version="$(extract_server_version "$repo_root/server.json")"
server_package_version="$(extract_server_package_version "$repo_root/server.json")"
formula_version="$(extract_formula_version "$repo_root/Formula/biomcp.rb")"

if [[ -z "$cargo_version" || -z "$python_version" || -z "$lock_version" || -z "$uv_lock_version" || -z "$manifest_version" || -z "$citation_version" || -z "$server_version" || -z "$server_package_version" || -z "$formula_version" ]]; then
    echo "Unable to read version from one or more manifests:" >&2
    echo "  Cargo.toml:              '$cargo_version'" >&2
    echo "  pyproject.toml:          '$python_version'" >&2
    echo "  Cargo.lock:              '$lock_version'" >&2
    echo "  uv.lock:                 '$uv_lock_version'" >&2
    echo "  manifest.json:           '$manifest_version'" >&2
    echo "  CITATION.cff:            '$citation_version'" >&2
    echo "  server.json:             '$server_version'" >&2
    echo "  server.json biomcp-cli:  '$server_package_version'" >&2
    echo "  Formula/biomcp.rb:       '$formula_version'" >&2
    exit 1
fi

ok=true
check_equal() {
    local expected_name="$1"
    local expected="$2"
    local name="$3"
    local actual="$4"
    if [[ "$expected" != "$actual" ]]; then
        echo "Version mismatch: $expected_name=$expected, $name=$actual" >&2
        ok=false
    fi
}

stable_pattern='(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)'
candidate_kind=""
stable_base=""
if [[ "$cargo_version" =~ ^${stable_pattern}$ ]]; then
    candidate_kind="release"
    stable_base="$cargo_version"
    check_equal "Cargo.toml" "$cargo_version" "pyproject.toml" "$python_version"
elif [[ "$cargo_version" =~ ^(${stable_pattern})-dev\.([1-9][0-9]*)$ ]]; then
    candidate_kind="development"
    stable_base="${BASH_REMATCH[1]}"
    expected_python_version="${stable_base}.dev${BASH_REMATCH[5]}"
    check_equal "Cargo.toml mapping" "$expected_python_version" "pyproject.toml" "$python_version"
else
    echo "Cargo.toml has a non-canonical release or development version: $cargo_version" >&2
    ok=false
fi

check_equal "Cargo.toml" "$cargo_version" "Cargo.lock" "$lock_version"
check_equal "pyproject.toml" "$python_version" "uv.lock" "$uv_lock_version"

if grep -qiE '^[[:space:]]*doi:[[:space:]]*.*(placeholder|xxxxxxx)' "$repo_root/CITATION.cff"; then
    echo "CITATION.cff contains a placeholder DOI" >&2
    ok=false
fi

if git -C "$repo_root" rev-parse --is-inside-work-tree >/dev/null 2>&1; then
    head_cargo_version="$(git -C "$repo_root" show HEAD:Cargo.toml 2>/dev/null | grep -m1 -E '^version\s*=\s*"' | sed -E 's/^[^"]*"([^"]+)".*$/\1/' || true)"
    head_python_version="$(git -C "$repo_root" show HEAD:pyproject.toml 2>/dev/null | grep -m1 -E '^version\s*=\s*"' | sed -E 's/^[^"]*"([^"]+)".*$/\1/' || true)"
    head_lock_version="$(git -C "$repo_root" show HEAD:Cargo.lock 2>/dev/null | awk '/name = "biomcp-cli"/{found=1} found && /^version/{print; exit}' | sed -E 's/^[^"]*"([^"]+)".*$/\1/' || true)"
    head_uv_lock_version="$(git -C "$repo_root" show HEAD:uv.lock 2>/dev/null | awk '/name = "biomcp-cli"/{found=1} found && /^version/{print; exit}' | sed -E 's/^[^"]*"([^"]+)".*$/\1/' || true)"
    if [[ "$cargo_version" != "$head_cargo_version" || "$python_version" != "$head_python_version" || "$lock_version" != "$head_lock_version" || "$uv_lock_version" != "$head_uv_lock_version" ]]; then
        echo "release version changes must be committed" >&2
        ok=false
    fi

    latest_tag="$(git -C "$repo_root" tag --merged HEAD --sort=version:refname --format='%(refname:short)' | grep -E '^v[0-9]+\.[0-9]+\.[0-9]+$' | tail -n1 || true)"
    if [[ -z "$latest_tag" ]]; then
        echo "a reachable release tag is required for the pre-1.0 boundary check" >&2
        ok=false
    else
        published_version="${latest_tag#v}"
        if [[ "$candidate_kind" == development ]]; then
            for entry in \
                "manifest.json:$manifest_version" \
                "CITATION.cff:$citation_version" \
                "server.json:$server_version" \
                "server.json biomcp-cli:$server_package_version"; do
                name="${entry%%:*}"
                actual="${entry#*:}"
                check_equal "latest stable tag" "$published_version" "$name" "$actual"
            done
            if [[ "$formula_version" != "__VERSION__" ]]; then
                check_equal "latest stable tag" "$published_version" "Formula/biomcp.rb" "$formula_version"
            fi
        else
            check_equal "Cargo.toml" "$cargo_version" "manifest.json" "$manifest_version"
            check_equal "Cargo.toml" "$cargo_version" "CITATION.cff" "$citation_version"
            check_equal "Cargo.toml" "$cargo_version" "server.json" "$server_version"
            check_equal "Cargo.toml" "$cargo_version" "server.json biomcp-cli" "$server_package_version"
            if [[ "$formula_version" != "__VERSION__" ]]; then
                check_equal "Cargo.toml" "$cargo_version" "Formula/biomcp.rb" "$formula_version"
            fi
        fi
        breaking_changes="$(awk '
            /^## Unreleased$/ { unreleased=1; next }
            unreleased && /^## / { exit }
            unreleased && /^### Breaking changes$/ { breaking=1; next }
            breaking && /^### / { exit }
            breaking && /^[[:space:]]*[-*][[:space:]]+[^[:space:]]/ { print; exit }
        ' "$repo_root/CHANGELOG.md")"
        if [[ -n "$breaking_changes" && "$stable_base" =~ ^([0-9]+)\.([0-9]+)\.([0-9]+)$ ]]; then
            proposed_major="${BASH_REMATCH[1]}"
            proposed_minor="${BASH_REMATCH[2]}"
            if [[ "$latest_tag" =~ ^v([0-9]+)\.([0-9]+)\.([0-9]+)$ ]]; then
                published_major="${BASH_REMATCH[1]}"
                published_minor="${BASH_REMATCH[2]}"
                if [[ "$published_major" == 0 && "$proposed_major" == 0 && "$proposed_minor" -le "$published_minor" ]]; then
                    echo "breaking changes require a minor version increase before 1.0" >&2
                    ok=false
                fi
            fi
        fi
    fi
fi

if [[ "$ok" == false ]]; then
    exit 1
fi

echo "Versions in sync: $cargo_version (Python $python_version; $candidate_kind candidate)"
