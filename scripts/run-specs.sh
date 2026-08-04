#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

SPEC_ROUTINE_PATHS=(
  spec/entity/article.md
  spec/entity/author.md
  spec/entity/disease-survival-fixture.md
  spec/entity/drug-interactions.md
  spec/entity/pgx.md
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
  spec/surface/skills.md
  tests/surface/test_parallel_isolation_contract.py
  spec/surface/cli-contract-ratchet.md
  spec/surface/build-profile.md
  spec/surface/trial-retirement.md
)

SPEC_STATIC_PATHS=(
  spec/surface/docker-image.md
  spec/surface/homebrew.md
)

SPEC_CTGOV_FIXTURE_PATHS=(
  spec/entity/trial-intervention-aliases.md
  spec/entity/trial-numeric-filters.md
  spec/entity/trial-documents.md
)

SPEC_LIVE_PATHS=(
  spec/entity/article-assets-live.md
  spec/entity/article-graph-live.md
  spec/entity/diagnostic.md
  spec/entity/disease.md
  spec/entity/drug.md
  spec/entity/gene.md
  spec/entity/pathway.md
  spec/entity/phenotype.md
  spec/entity/protein.md
  spec/entity/trial.md
  spec/entity/vaers.md
  spec/entity/variant-hotspots.md
  spec/entity/clingen-erepo-live.md
  spec/entity/clingen-cspec-live.md
  spec/entity/clingen-car-live.md
  spec/entity/clingen-ldh-live.md
  spec/entity/variant-myvariant-live.md
  spec/entity/variant-articles-live.md
  spec/surface/cli.md
  spec/surface/discover.md
)

usage() {
  echo "usage: scripts/run-specs.sh <spec|spec-static|spec-pr|spec-contracts|verify|verify-nih-reporter>" >&2
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

paths_include_any() {
  local candidate path
  for candidate in "$@"; do
    for path in "${paths[@]}"; do
      [[ "$path" == "$candidate" ]] && return 0
    done
  done
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
  local env_file="$ROOT/.cache/spec-ctgov-intervention-alias-env"
  rm -f "$env_file"
  unset BIOMCP_CTGOV_BASE BIOMCP_CTGOV_CDN_BASE
  bash spec/fixtures/setup-ctgov-intervention-alias-spec-fixture.sh "$ROOT"
  register_cleanup cleanup_ctgov_fixture
  [[ -s "$env_file" ]] || {
    echo "CTGov fixture did not create $env_file" >&2
    return 1
  }
  # shellcheck source=/dev/null
  . "$env_file"
}

require_ctgov_fixture_env() {
  : "${BIOMCP_CTGOV_BASE:?CTGov fixture did not export BIOMCP_CTGOV_BASE}"
  : "${BIOMCP_CTGOV_CDN_BASE:?CTGov fixture did not export BIOMCP_CTGOV_CDN_BASE}"
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

cleanup_clingen_cspec_fixture() {
  bash spec/fixtures/cleanup-clingen-cspec-spec-fixture.sh "$ROOT"
}

cleanup_cpic_fixture() {
  bash spec/fixtures/cleanup-cpic-spec-fixture.sh "$ROOT"
}

run_cpic_fixture() {
  # Isolated runner-lifecycle tests copy only the fixture subset they exercise.
  [[ -x spec/fixtures/setup-cpic-spec-fixture.sh ]] || return 0
  bash spec/fixtures/setup-cpic-spec-fixture.sh "$ROOT"
  source_if_present "$ROOT/.cache/spec-cpic-env"
  register_cleanup cleanup_cpic_fixture
}

run_clingen_cspec_fixture() {
  # Isolated runner-lifecycle tests copy the historical fixture subset; the real
  # routine workspace always includes this ticket's CSpec fixture.
  if [[ ! -x spec/fixtures/setup-clingen-cspec-spec-fixture.sh ]]; then
    return
  fi
  bash spec/fixtures/setup-clingen-cspec-spec-fixture.sh "$ROOT"
  source_if_present "$ROOT/.cache/spec-clingen-cspec-env"
  register_cleanup cleanup_clingen_cspec_fixture
}

reap_stale_routine_fixtures() {
  local cleanup lock_path="$ROOT/.cache/spec-routine-fixtures.lock"
  mkdir -p "$ROOT/.cache"
  : >"$lock_path"
  for cleanup in \
    cleanup-article-fulltext-source-fixture.sh \
    cleanup-ctgov-intervention-alias-spec-fixture.sh \
    cleanup-disease-survival-spec-fixture.sh \
    cleanup-variant-identity-spec-fixture.sh \
    cleanup-clingen-cspec-spec-fixture.sh \
    cleanup-cpic-spec-fixture.sh; do
    [[ -x "spec/fixtures/$cleanup" ]] || continue
    ROUTINE_FIXTURE_LOCK_PATH="$lock_path" bash "spec/fixtures/$cleanup" "$ROOT"
  done
}

lock_routine_fixtures() {
  mkdir -p "$ROOT/.cache"
  exec 8>"$ROOT/.cache/spec-routine-fixtures.lock"
  # Bounded wait. flock binds to the open file description, not the PID, so a
  # background fixture server that inherited fd 8 and outlived an interrupted
  # run holds this lock forever. Every fixture now closes fd 8 explicitly, but
  # if one ever leaks again this must fail loudly in minutes rather than hang
  # until the caller's own timeout kills it hours later.
  if ! flock -w 300 8; then
    printf 'error: could not acquire %s within 300s.\n' \
      "$ROOT/.cache/spec-routine-fixtures.lock" >&2
    printf 'A leaked fixture process is probably still holding it. Find it with:\n' >&2
    printf '  for f in /proc/[0-9]*/fd/*; do readlink "$f" | grep -q spec-routine-fixtures.lock && echo "$f"; done\n' >&2
    exit 1
  fi
}

run_section_outcome_specs() {
  if ((${#SECTION_OUTCOME_MD_PATHS[@]})); then
    (
      bash spec/fixtures/setup-section-outcomes-spec-fixture.sh "$ROOT"
      source_if_present "$ROOT/.cache/spec-section-outcomes-env"
      trap 'bash spec/fixtures/cleanup-section-outcomes-spec-fixture.sh "$ROOT"' EXIT
      8>&- mustmatch test "${SECTION_OUTCOME_MD_PATHS[@]}" --lang bash "${timeout_args[@]}"
    )
  fi
}

run_article_markdown_specs() {
  if ((${#ARTICLE_MD_PATHS[@]})); then
    (
      unset BIOMCP_CACHE_MODE
      source_if_present "$ROOT/.cache/spec-article-fulltext-source-env"
      8>&- mustmatch test "${ARTICLE_MD_PATHS[@]}" --lang bash "${timeout_args[@]}"
    )
  fi
}

run_markdown_specs() {
  if ((${#MD_PATHS[@]})); then
    8>&- mustmatch test "${MD_PATHS[@]}" --lang bash "${timeout_args[@]}"
  fi
}

run_python_contracts() {
  if ((${#PY_PATHS[@]})); then
    8>&- uv run --with pytest --no-project pytest "${PY_PATHS[@]}" -v
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
    reap_stale_routine_fixtures
    lock_routine_fixtures
    run_article_fixture
    run_study_fixture
    run_ddinter_fixture
    if paths_include_any "${SPEC_CTGOV_FIXTURE_PATHS[@]}"; then
      run_ctgov_fixture
      require_ctgov_fixture_env
    fi
    run_disease_survival_fixture
    run_variant_identity_fixture
    run_clingen_cspec_fixture
    run_cpic_fixture
    ;;
  spec-static)
    timeout_args=(--timeout 180)
    paths=("${SPEC_STATIC_PATHS[@]}")
    mustmatch_path_dir="$(mustmatch_dir)"
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
    reap_stale_routine_fixtures
    lock_routine_fixtures
    run_article_fixture
    run_study_fixture
    if paths_include_any "${SPEC_CTGOV_FIXTURE_PATHS[@]}"; then
      run_ctgov_fixture
      require_ctgov_fixture_env
    fi
    ;;
  verify)
    timeout_args=(--timeout 180)
    paths=(
      spec/entity/article-assets-live.md
      spec/entity/article-graph-live.md
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
      spec/entity/clingen-car-live.md
      spec/entity/clingen-ldh-live.md
      spec/entity/variant-myvariant-live.md
      spec/entity/variant-articles-live.md
      spec/surface/build-profile-live.md
      spec/surface/cli.md
      spec/surface/discover.md
    )
    mustmatch_path_dir="$(mustmatch_dir)"
    run_live_ddinter_root
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

if [[ "$mode" == "spec-static" ]]; then
  export PATH="$mustmatch_path_dir:$PATH"
else
  case "$mode" in
    verify) default_biomcp_bin="$ROOT/target/release/biomcp" ;;
    verify-nih-reporter) default_biomcp_bin="$ROOT/target/release/biomcp" ;;
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
fi

if [[ "$mode" == verify* ]]; then
  prebuild_cargo_test_targets
fi

partition_paths "${paths[@]}"
run_article_markdown_specs
run_markdown_specs
run_section_outcome_specs
run_python_contracts
