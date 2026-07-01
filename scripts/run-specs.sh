#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

SPEC_ROUTINE_PATHS=(
  spec/entity/article.md
  spec/entity/study.md
  spec/entity/variant.md
  spec/surface/mcp.md
  spec/surface/skills.md
  tests/surface/test_parallel_isolation_contract.py
  spec/surface/cli-contract-ratchet.md
  spec/surface/trial-action-summary.md
  spec/surface/ctgov-helper-pivots.md
)

SPEC_LIVE_PATHS=(
  spec/entity/diagnostic.md
  spec/entity/disease.md
  spec/entity/drug.md
  spec/entity/gene.md
  spec/entity/pathway.md
  spec/entity/pgx.md
  spec/entity/phenotype.md
  spec/entity/protein.md
  spec/entity/trial.md
  spec/entity/vaers.md
  spec/entity/variant-hotspots.md
  spec/surface/cli.md
  spec/surface/discover.md
)

usage() {
  echo "usage: scripts/run-specs.sh <spec|spec-pr|spec-contracts|verify|verify-cpic|verify-nih-reporter>" >&2
}

mustmatch_dir() {
  local candidate version
  for candidate in "${MUSTMATCH_BIN:-}" "$HOME/.local/bin/mustmatch" "$(command -v mustmatch 2>/dev/null || true)"; do
    if [[ -n "$candidate" && -x "$candidate" ]]; then
      version="$("$candidate" --version 2>/dev/null || true)"
      case "$version" in
        "mustmatch 0.0.4"*) ;;
        "mustmatch "*) dirname "$candidate"; return 0 ;;
      esac
    fi
  done
  echo "standalone mustmatch binary not found on PATH or at ~/.local/bin/mustmatch" >&2
  return 1
}

partition_paths() {
  MD_PATHS=()
  PY_PATHS=()
  local path
  for path in "$@"; do
    case "$path" in
      *.md) MD_PATHS+=("$path") ;;
      *.py) PY_PATHS+=("$path") ;;
      *) echo "unsupported spec path extension: $path" >&2; return 1 ;;
    esac
  done
}

source_if_present() {
  local path="$1"
  if [[ -f "$path" ]]; then
    # shellcheck source=/dev/null
    . "$path"
  fi
}

run_study_fixture() {
  bash spec/fixtures/setup-study-spec-fixture.sh "$ROOT"
  source_if_present "$ROOT/.cache/spec-study-env"
}

run_ddinter_fixture() {
  bash spec/fixtures/setup-ddinter-spec-fixture.sh "$ROOT"
  source_if_present "$ROOT/.cache/spec-ddinter-env"
}

run_ctgov_fixture() {
  bash spec/fixtures/setup-ctgov-intervention-alias-spec-fixture.sh "$ROOT"
  source_if_present "$ROOT/.cache/spec-ctgov-intervention-alias-env"
}

run_markdown_specs() {
  if ((${#MD_PATHS[@]})); then
    mustmatch test "${MD_PATHS[@]}" --lang bash "${timeout_args[@]}"
  fi
}

run_python_contracts() {
  if ((${#PY_PATHS[@]})); then
    uv run --with pytest --no-project pytest "${PY_PATHS[@]}" -v
  fi
}

prebuild_cargo_test_targets() {
  echo "run-specs: pre-building cargo test binaries ($*) for live specs" >&2
  cargo test --locked --no-run "$@"
}

mode="${1:-}"
case "$mode" in
  spec|spec-pr)
    timeout_args=(--timeout 180)
    paths=("${SPEC_ROUTINE_PATHS[@]}")
    mustmatch_path_dir="$(mustmatch_dir)"
    run_study_fixture
    run_ddinter_fixture
    run_ctgov_fixture
    ;;
  spec-contracts)
    timeout_args=(--timeout 180)
    paths=(
      spec/entity/article.md
      spec/surface/mcp.md
      spec/surface/skills.md
      spec/surface/trial-action-summary.md
    )
    mustmatch_path_dir="$(mustmatch_dir)"
    run_study_fixture
    run_ctgov_fixture
    ;;
  verify)
    timeout_args=(--timeout 180)
    paths=(
      spec/entity/diagnostic.md
      spec/entity/drug.md
      spec/entity/pathway.md
      spec/entity/phenotype.md
      spec/entity/protein.md
      spec/entity/trial.md
      spec/entity/vaers.md
      spec/entity/variant-hotspots.md
      spec/surface/cli.md
      spec/surface/discover.md
    )
    mustmatch_path_dir="$(mustmatch_dir)"
    ;;
  verify-cpic)
    timeout_args=(--timeout 180)
    paths=(spec/entity/pgx.md)
    mustmatch_path_dir="$(mustmatch_dir)"
    ;;
  verify-nih-reporter)
    timeout_args=(--timeout 180)
    paths=(spec/entity/disease.md spec/entity/gene.md)
    mustmatch_path_dir="$(mustmatch_dir)"
    ;;
  *)
    usage
    exit 2
    ;;
esac

case "$mode" in
  verify) default_biomcp_bin="$ROOT/target/release/biomcp" ;;
  verify-cpic|verify-nih-reporter) default_biomcp_bin="$ROOT/target/release/biomcp" ;;
  *) default_biomcp_bin="$ROOT/target/spec/biomcp" ;;
esac
BIOMCP_BIN="${BIOMCP_BIN:-$default_biomcp_bin}"
case "$BIOMCP_BIN" in
  /*) ;;
  *) BIOMCP_BIN="$ROOT/$BIOMCP_BIN" ;;
esac
BIOMCP_BIN_DIR="$(cd "$(dirname "$BIOMCP_BIN")" && pwd)"
export BIOMCP_BIN
export PATH="$BIOMCP_BIN_DIR:$mustmatch_path_dir:$PATH"

if [[ "$mode" == verify* ]]; then
  prebuild_cargo_test_targets
fi

partition_paths "${paths[@]}"
run_markdown_specs
run_python_contracts
