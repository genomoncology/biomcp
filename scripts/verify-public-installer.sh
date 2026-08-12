#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PUBLIC_URL="${BIOMCP_PUBLIC_INSTALLER_URL:-https://biomcp.org/install.sh}"
scratch="$(mktemp -d)"
cleanup() { rm -rf "$scratch"; }
trap cleanup EXIT

curl -fsSL "$PUBLIC_URL" -o "$scratch/install.sh"
if ! cmp --silent "$ROOT/install.sh" "$scratch/install.sh"; then
  echo "Deployed installer differs from canonical install.sh: $PUBLIC_URL" >&2
  exit 1
fi
echo "Public installer matches canonical install.sh"
