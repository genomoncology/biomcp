#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BIN="${BIOMCP_BIN:-$ROOT/target/release/biomcp}"
if [[ ! -x "$BIN" ]]; then
  if command -v biomcp >/dev/null 2>&1; then
    BIN="$(command -v biomcp)"
  fi
fi

if [[ ! -x "$BIN" ]]; then
  echo "biomcp binary not found; set BIOMCP_BIN or build ./target/release/biomcp" >&2
  exit 1
fi

# Keep structured command errors visible when stdout feeds a JSON variable.
command_json() {
  local output status
  if output=$("$BIN" "$@"); then
    printf '%s\n' "$output"
  else
    status=$?
    printf '%s\n' "$output" >&2
    return "$status"
  fi
}

echo "GeneGPT demo: BRAF V600E evidence walk"

gene_json="$(command_json --json get gene BRAF)"
variant_json="$(command_json --json get variant "BRAF V600E" population)"
trial_json="$(command_json --json variant trials "BRAF V600E" --limit 3)"
article_json="$(command_json --json search article -g BRAF -d melanoma --limit 3)"

if ! grep -q '"symbol"' <<<"$gene_json"; then
  echo "Gene lookup failed to return symbol" >&2
  exit 1
fi
if ! grep -q '"population"' <<<"$variant_json"; then
  echo "Variant lookup failed to return population section" >&2
  exit 1
fi

trial_count="$(sed -n 's/.*"count":[[:space:]]*\([0-9][0-9]*\).*/\1/p' <<<"$trial_json" | head -n1)"
article_count="$(sed -n 's/.*"count":[[:space:]]*\([0-9][0-9]*\).*/\1/p' <<<"$article_json" | head -n1)"
trial_count="${trial_count:-0}"
article_count="${article_count:-0}"
evidence_score=$((trial_count + article_count))

if [[ $evidence_score -le 0 ]]; then
  echo "Evidence score was zero; expected at least one supporting trial/article hit" >&2
  exit 1
fi

echo "GeneGPT evidence score: $evidence_score"

echo "GeneGPT demo complete"
