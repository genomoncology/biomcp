#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BIN="${BIOMCP_BIN:-$ROOT/target/debug/biomcp}"
if [[ ! -x "$BIN" ]]; then
  BIN="biomcp"
fi

echo "GeneAgent demo: variant, pathway, and protein pivots"

variant_json="$("$BIN" --json get variant "BRAF V600E" clinvar)"
pathway_json="$("$BIN" --json get pathway R-HSA-5673001 genes)"
drug_json="$("$BIN" --json pathway drugs R-HSA-5673001 --limit 3)"
protein_json="$("$BIN" --json protein structures P15056)"

if ! grep -q '"id"' <<<"$variant_json" || ! grep -q '"gene"' <<<"$variant_json"; then
  echo "Variant lookup failed" >&2
  exit 1
fi
if ! grep -q '"genes"' <<<"$pathway_json"; then
  echo "Pathway lookup failed to include genes" >&2
  exit 1
fi
if ! grep -q '"structures"' <<<"$protein_json"; then
  echo "Protein helper failed to include structures" >&2
  exit 1
fi

drug_count="$(sed -n 's/.*"count":[[:space:]]*\([0-9][0-9]*\).*/\1/p' <<<"$drug_json" | head -n1)"
drug_count="${drug_count:-0}"
if [[ $drug_count -le 0 ]]; then
  echo "No pathway-linked drugs were returned for demo scoring" >&2
  exit 1
fi

echo "GeneAgent reproduction score: $drug_count"

echo "GeneAgent demo complete"
