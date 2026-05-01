#!/usr/bin/env bash
set -euo pipefail

repo_root="${1:-$PWD}"
cache_dir="$repo_root/.cache"
ddinter_root="$cache_dir/spec-ddinter"

rm -rf "$ddinter_root"
mkdir -p "$ddinter_root" "$cache_dir"

header='DDInterID_A,Drug_A,DDInterID_B,Drug_B,Level'
cat >"$ddinter_root/ddinter_downloads_code_A.csv" <<'EOF'
DDInterID_A,Drug_A,DDInterID_B,Drug_B,Level
DDInterW,Warfarin,DDInterAMX,Amoxicillin,Major
DDInterW,Warfarin,DDInterATO,Atorvastatin,Moderate
DDInterW,Warfarin,DDInterCLO,Clopidogrel,Major
DDInterI,Imatinib,DDInterCYP,CYP3A4 inhibitor,Major
EOF

for code in B D H L P R V; do
  printf '%s\n' "$header" >"$ddinter_root/ddinter_downloads_code_${code}.csv"
done

printf 'export BIOMCP_DDINTER_DIR=%q\n' "$ddinter_root" >"$cache_dir/spec-ddinter-env"
