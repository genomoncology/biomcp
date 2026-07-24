#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

SPEC_ROUTINE_PATHS=(
  spec/entity/article.md
  spec/entity/author.md
  spec/entity/disease-survival-fixture.md
  spec/entity/drug-interactions.md
  spec/entity/gwas-numeric-filters.md
  spec/entity/section-outcomes.md
  spec/entity/study.md
  spec/entity/trial-intervention-aliases.md
  spec/entity/trial-numeric-filters.md
  spec/entity/trial-documents.md
  spec/entity/variant.md
  spec/entity/clingen-erepo.md
  spec/entity/clingen-cspec.md
  spec/entity/variant-article-identity.md
  spec/surface/mcp.md
  spec/surface/discover-input.md
  spec/surface/docker-image.md
  spec/surface/homebrew.md
  spec/surface/skills.md
  tests/surface/test_parallel_isolation_contract.py
  spec/surface/cli-contract-ratchet.md
  spec/surface/trial-retirement.md
  spec/surface/ctgov-helper-pivots.md
)

SPEC_LIVE_PATHS=(
  spec/entity/article-assets-live.md
  spec/entity/article-graph-live.md
  spec/entity/article-indexing-live.md
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
  spec/entity/clingen-erepo-live.md
  spec/entity/clingen-cspec-live.md
  spec/entity/variant-myvariant-live.md
  spec/entity/variant-articles-live.md
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
  ARTICLE_MD_PATHS=()
  SECTION_OUTCOME_MD_PATHS=()
  MD_PATHS=()
  PY_PATHS=()
  local path
  for path in "$@"; do
    case "$path" in
      spec/entity/article.md|spec/entity/author.md) ARTICLE_MD_PATHS+=("$path") ;;
      spec/entity/section-outcomes.md) SECTION_OUTCOME_MD_PATHS+=("$path") ;;
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

CLEANUP_FUNCTIONS=()

register_cleanup() {
  CLEANUP_FUNCTIONS+=("$1")
}

cleanup_all() {
  local exit_status=$?
  local cleanup_function
  set +e
  for cleanup_function in "${CLEANUP_FUNCTIONS[@]}"; do
    "$cleanup_function"
  done
  trap - EXIT
  exit "$exit_status"
}

handle_signal() {
  local signal_number="$1"
  exit "$((128 + signal_number))"
}

trap cleanup_all EXIT
trap 'handle_signal 2' INT
trap 'handle_signal 15' TERM
trap 'handle_signal 1' HUP

run_study_fixture() {
  bash spec/fixtures/setup-study-spec-fixture.sh "$ROOT"
  source_if_present "$ROOT/.cache/spec-study-env"
}

run_ddinter_fixture() {
  bash spec/fixtures/setup-ddinter-spec-fixture.sh "$ROOT"
  source_if_present "$ROOT/.cache/spec-ddinter-env"
}

run_live_ddinter_root() {
  export BIOMCP_DDINTER_DIR="$ROOT/.cache/verify-ddinter-live"
  rm -rf "$BIOMCP_DDINTER_DIR"
}

cleanup_article_fixture() {
  bash spec/fixtures/cleanup-article-fulltext-source-fixture.sh "$ROOT"
}

run_article_fixture() {
  bash spec/fixtures/setup-article-fulltext-source-fixture.sh "$ROOT"
  register_cleanup cleanup_article_fixture
}

cleanup_ctgov_fixture() {
  bash spec/fixtures/cleanup-ctgov-intervention-alias-spec-fixture.sh "$ROOT"
}

run_ctgov_fixture() {
  bash spec/fixtures/setup-ctgov-intervention-alias-spec-fixture.sh "$ROOT"
  source_if_present "$ROOT/.cache/spec-ctgov-intervention-alias-env"
  register_cleanup cleanup_ctgov_fixture
}

cleanup_disease_survival_fixture() {
  bash spec/fixtures/cleanup-disease-survival-spec-fixture.sh "$ROOT"
}

run_disease_survival_fixture() {
  bash spec/fixtures/setup-disease-survival-spec-fixture.sh "$ROOT"
  source_if_present "$ROOT/.cache/spec-disease-survival-env"
  register_cleanup cleanup_disease_survival_fixture
}

cleanup_variant_identity_fixture() {
  bash spec/fixtures/cleanup-variant-identity-spec-fixture.sh "$ROOT"
}

run_variant_identity_fixture() {
  bash spec/fixtures/setup-variant-identity-spec-fixture.sh "$ROOT"
  source_if_present "$ROOT/.cache/spec-variant-identity-env"
  register_cleanup cleanup_variant_identity_fixture
}

lock_routine_fixtures() {
  mkdir -p "$ROOT/.cache"
  exec 8>"$ROOT/.cache/spec-routine-fixtures.lock"
  flock 8
}

run_section_outcome_specs() {
  if ((${#SECTION_OUTCOME_MD_PATHS[@]})); then
    (
      bash spec/fixtures/setup-section-outcomes-spec-fixture.sh "$ROOT"
      source_if_present "$ROOT/.cache/spec-section-outcomes-env"
      trap 'bash spec/fixtures/cleanup-section-outcomes-spec-fixture.sh "$ROOT"' EXIT
      mustmatch test "${SECTION_OUTCOME_MD_PATHS[@]}" --lang bash "${timeout_args[@]}"
    )
  fi
}

run_article_markdown_specs() {
  if ((${#ARTICLE_MD_PATHS[@]})); then
    (
      unset BIOMCP_CACHE_MODE
      source_if_present "$ROOT/.cache/spec-article-fulltext-source-env"
      mustmatch test "${ARTICLE_MD_PATHS[@]}" --lang bash "${timeout_args[@]}"
    )
  fi
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
    lock_routine_fixtures
    run_article_fixture
    run_study_fixture
    run_ddinter_fixture
    run_ctgov_fixture
    run_disease_survival_fixture
    run_variant_identity_fixture
    ;;
  spec-contracts)
    timeout_args=(--timeout 180)
    paths=(
      spec/entity/article.md
      spec/entity/author.md
      spec/surface/mcp.md
      spec/surface/skills.md
      spec/surface/trial-retirement.md
    )
    mustmatch_path_dir="$(mustmatch_dir)"
    lock_routine_fixtures
    run_article_fixture
    run_study_fixture
    run_ctgov_fixture
    ;;
  verify)
    timeout_args=(--timeout 180)
    paths=(
      spec/entity/article-assets-live.md
      spec/entity/article-graph-live.md
      spec/entity/article-indexing-live.md
      spec/entity/diagnostic.md
      spec/entity/drug.md
      spec/entity/ddinter-live.md
      spec/entity/pathway.md
      spec/entity/phenotype.md
      spec/entity/protein.md
      spec/entity/trial.md
      spec/entity/vaers.md
      spec/entity/variant-hotspots.md
      spec/entity/clingen-erepo-live.md
      spec/entity/clingen-cspec-live.md
      spec/entity/variant-myvariant-live.md
      spec/entity/variant-articles-live.md
      spec/surface/cli.md
      spec/surface/discover.md
    )
    mustmatch_path_dir="$(mustmatch_dir)"
    run_live_ddinter_root
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

if [[ -n "${BIOMCP_SPEC_RUNNER_READY_FILE:-}" ]]; then
  : >"$BIOMCP_SPEC_RUNNER_READY_FILE"
fi
if [[ "${BIOMCP_SPEC_RUNNER_HOLD:-0}" == 1 ]]; then
  while :; do
    sleep 1
  done
fi

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
run_article_markdown_specs
run_markdown_specs
run_section_outcome_specs
run_python_contracts
