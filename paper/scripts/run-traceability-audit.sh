#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

discover_biomcp() {
  if [[ -n "${BIOMCP_BIN:-}" && -x "${BIOMCP_BIN}" ]]; then
    printf '%s\n' "${BIOMCP_BIN}"
    return 0
  fi

  if [[ -x "$ROOT/target/release/biomcp" ]]; then
    printf '%s\n' "$ROOT/target/release/biomcp"
    return 0
  fi

  if command -v biomcp >/dev/null 2>&1; then
    command -v biomcp
    return 0
  fi

  return 1
}

if ! BIN="$(discover_biomcp)"; then
  echo "biomcp binary not found; set BIOMCP_BIN, build ./target/release/biomcp, or install biomcp on PATH" >&2
  exit 1
fi

DEFAULT_OUTPUT_DIR="$ROOT/paper/generated/traceability"
OUT_DIR="${1:-$DEFAULT_OUTPUT_DIR}"
mkdir -p "$OUT_DIR"

printf 'entity\tsample_id\toutput_path\tcommand\n' > "$OUT_DIR/manifest.tsv"

capture() {
  local entity="$1"
  local sample_id="$2"
  local filename="$3"
  shift 3
  local output_path="$OUT_DIR/$filename"
  "$BIN" --json "$@" > "$output_path"
  printf '%s\t%s\t%s\t%s\n' \
    "$entity" \
    "$sample_id" \
    "$output_path" \
    "$BIN --json $*" >> "$OUT_DIR/manifest.tsv"
}

capture "gene" "CFTR" "gene-cftr.json" get gene CFTR all
capture "variant" "rs334" "variant-rs334.json" get variant rs334 all
capture "trial" "NCT06668103" "trial-nct06668103.json" get trial NCT06668103
capture "article" "22663011" "article-22663011.json" get article 22663011
capture "disease" "cystic fibrosis" "disease-cystic-fibrosis.json" get disease "cystic fibrosis" all
capture "drug" "ivacaftor" "drug-ivacaftor.json" get drug ivacaftor all
capture "pathway" "R-HSA-5358351" "pathway-r-hsa-5358351.json" get pathway R-HSA-5358351 all
capture "pgx" "CYP2D6" "pgx-cyp2d6.json" get pgx CYP2D6 all
capture "adverse-event" "10329882" "adverse-event-10329882.json" get adverse-event 10329882 all

echo "Wrote sampled traceability captures to $OUT_DIR"
