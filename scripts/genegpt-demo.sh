#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BIN="${BIOMCP_BIN:-$ROOT/target/debug/biomcp}"
if [[ ! -x "$BIN" ]]; then
  BIN="biomcp"
fi

echo "GeneGPT demo: BRAF V600E evidence walk"

gene_json="$("$BIN" --json get gene BRAF)"
variant_json="$("$BIN" --json get variant "BRAF V600E" population)"
trial_json="$("$BIN" --json variant trials "BRAF V600E" --limit 3)"
article_json="$("$BIN" --json search article -g BRAF -d melanoma --limit 3)"

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
