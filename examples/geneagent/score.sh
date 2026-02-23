#!/usr/bin/env bash
set -euo pipefail

OUT="${1:-output.md}"
TOTAL=0
FOUND=0

check() {
  TOTAL=$((TOTAL + 1))
  if grep -Eiq "$1" "$OUT"; then
    FOUND=$((FOUND + 1))
    echo "  PASS: $1"
  else
    echo "  MISS: $1"
  fi
}

check "BRAF"
check "V600E|variant"
check "pathway|MAPK|RAS"
check "drug|therapy|inhibitor"
check "protein|P15056|kinase"

echo "Correctness: $FOUND/$TOTAL"
