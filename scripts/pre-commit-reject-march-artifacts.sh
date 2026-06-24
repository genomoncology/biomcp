#!/usr/bin/env bash
set -euo pipefail

repo_root="$(git rev-parse --show-toplevel)"
cd "$repo_root"

staged_paths_file="$(mktemp)"
trap 'rm -f "$staged_paths_file"' EXIT

git diff --cached --name-only -z --diff-filter=ACMRT -- .march >"$staged_paths_file"

offending_paths=()
while IFS= read -r -d '' path; do
    case "$path" in
        .march/code-review-log.md)
            ;;
        .march/*)
            offending_paths+=("$path")
            ;;
    esac
done <"$staged_paths_file"

if (( ${#offending_paths[@]} > 0 )); then
    echo "Error: staged non-allowlisted .march artifacts detected:" >&2
    for path in "${offending_paths[@]}"; do
        printf '  - %q\n' "$path" >&2
    done
    echo "Allowed .march paths:" >&2
    echo "  - .march/code-review-log.md" >&2
    echo "Remediation:" >&2
    echo "  - Unstage local March artifacts: git restore --staged -- <path>" >&2
    echo "  - Remove already tracked artifacts: git rm --cached -- <path>" >&2
    exit 1
fi
