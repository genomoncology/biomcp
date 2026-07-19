#!/usr/bin/env bash
set -euo pipefail

query="$(printf 'x%.0s' {1..4097})"
exec biomcp --json discover "$query"
