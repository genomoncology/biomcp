#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

routine_owner_stat="$(<"/proc/$$/stat")"
routine_owner_rest="${routine_owner_stat#*) }"
read -r -a routine_owner_fields <<<"$routine_owner_rest"
export ROUTINE_FIXTURE_OWNER_PID="$$"
export ROUTINE_FIXTURE_OWNER_START_ID="${routine_owner_fields[19]}"

SPEC_ROUTINE_PATHS=(
  spec/entity/article.md
  spec/entity/author.md
  spec/entity/disease.md
  spec/entity/disease-survival-fixture.md
  spec/entity/phenotype.md
  spec/surface/discover.md
  spec/entity/diagnostic.md
  spec/entity/vaers.md
  spec/entity/pathway.md
  spec/entity/trial.md
  spec/entity/drug.md
  spec/entity/gene.md
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
  spec/entity/variant-articles-corpus.md
  spec/entity/protein.md
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
  spec/surface/trial-retirement.md
)

SPEC_LIVE_PATHS=(
  spec/entity/article-assets-live.md
  spec/entity/article-graph-live.md
  spec/entity/ddinter-live.md
  spec/entity/disease-live.md
  spec/entity/variant-hotspots.md
  spec/entity/variant-myvariant-live.md
  spec/entity/variant-articles-live.md
  spec/surface/build-profile-live.md
  spec/surface/cli.md
  spec/surface/discover-live.md
)

SPEC_NIH_REPORTER_LIVE_PATHS=(
  spec/entity/nih-reporter-live.md
)

usage() {
  echo "usage: scripts/run-specs.sh <prepare-spec|spec|spec-static|spec-pr|spec-contracts|verify|verify-nih-reporter>" >&2
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
      # These pages share one article server and its mutable request log, so
      # Mustmatch retains their declared order in one serial invocation.
      spec/entity/article.md|spec/entity/author.md) ARTICLE_MD_PATHS+=("$path") ;;
      # This page owns a separate setup/cleanup subshell and generated inputs.
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
PARALLEL_SPEC_PIDS=()
PARALLEL_SPEC_OUTPUT_DIR=""

register_cleanup() {
  CLEANUP_FUNCTIONS+=("$1")
}

cleanup_parallel_spec_workers() {
  local pid
  for pid in "${PARALLEL_SPEC_PIDS[@]}"; do
    kill -TERM -- "-$pid" 2>/dev/null || true
  done
  if ((${#PARALLEL_SPEC_PIDS[@]})); then
    sleep 0.2
  fi
  for pid in "${PARALLEL_SPEC_PIDS[@]}"; do
    kill -KILL -- "-$pid" 2>/dev/null || true
  done
  for pid in "${PARALLEL_SPEC_PIDS[@]}"; do
    wait "$pid" 2>/dev/null || true
  done
  PARALLEL_SPEC_PIDS=()
  if [[ -n "$PARALLEL_SPEC_OUTPUT_DIR" && -d "$PARALLEL_SPEC_OUTPUT_DIR" ]]; then
    rm -r "$PARALLEL_SPEC_OUTPUT_DIR"
  fi
  PARALLEL_SPEC_OUTPUT_DIR=""
}

register_cleanup cleanup_parallel_spec_workers

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

cleanup_provider_contract_fixture() {
  bash spec/fixtures/cleanup-provider-contract-spec-fixture.sh "$ROOT"
}

run_provider_contract_fixture() {
  [[ -x spec/fixtures/setup-provider-contract-spec-fixture.sh ]] || return 0
  bash spec/fixtures/setup-provider-contract-spec-fixture.sh "$ROOT"
  source_if_present "$ROOT/.cache/spec-provider-contract-env"
  register_cleanup cleanup_provider_contract_fixture
}

cleanup_protein_fixture() {
  bash spec/fixtures/cleanup-complexportal-spec-fixture.sh "$ROOT"
}

run_protein_fixture() {
  # Runner lifecycle tests intentionally copy only the fixture subset they exercise.
  [[ -x spec/fixtures/setup-complexportal-spec-fixture.sh ]] || return 0
  bash spec/fixtures/setup-complexportal-spec-fixture.sh "$ROOT"
  source_if_present "$ROOT/.cache/spec-complexportal-env"
  register_cleanup cleanup_protein_fixture
}

cleanup_vaers_fixture() {
  bash spec/fixtures/cleanup-vaers-spec-fixture.sh "$ROOT"
}

run_vaers_fixture() {
  # Small runner-lifecycle tests copy only the fixture subset they exercise.
  [[ -x spec/fixtures/setup-vaers-spec-fixture.sh ]] || return 0
  bash spec/fixtures/setup-vaers-spec-fixture.sh "$ROOT"
  source_if_present "$ROOT/.cache/spec-vaers-env"
  register_cleanup cleanup_vaers_fixture
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
    cleanup-provider-contract-spec-fixture.sh \
    cleanup-vaers-spec-fixture.sh \
    cleanup-variant-identity-spec-fixture.sh \
    cleanup-clingen-cspec-spec-fixture.sh \
    cleanup-cpic-spec-fixture.sh \
    cleanup-complexportal-spec-fixture.sh; do
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

validate_routine_spec_workers() {
  SPEC_WORKER_COUNT="${BIOMCP_SPEC_WORKERS:-4}"
  if [[ ! "$SPEC_WORKER_COUNT" =~ ^[1-9][0-9]*$ ]]; then
    echo "BIOMCP_SPEC_WORKERS must be a positive integer, got: $SPEC_WORKER_COUNT" >&2
    return 2
  fi
}

ctgov_page_consumes_request_log() {
  local path="$1"
  grep -Fq 'BIOMCP_CTGOV_INTERVENTION_ALIAS_REQUEST_LOG' "$path" ||
    grep -Fq 'spec/fixtures/ctgov-request-log' "$path"
}

prepare_ctgov_page_request_log() {
  local fixture_root="${BIOMCP_CTGOV_INTERVENTION_ALIAS_ROOT:?CTGov fixture root is not configured}"
  local fixture_base="${BIOMCP_CTGOV_BASE:?CTGov fixture base is not configured}"
  local request_log namespace
  request_log="$(mktemp "$fixture_root/request-log.XXXXXX")"
  namespace="${request_log##*/}"
  export BIOMCP_CTGOV_INTERVENTION_ALIAS_REQUEST_LOG="$request_log"
  export BIOMCP_CTGOV_BASE="${fixture_base%/api/v2}/__biomcp_ctgov_worker/$namespace/api/v2"
}

run_markdown_specs() {
  ((${#MD_PATHS[@]})) || return 0

  case "$mode" in
    spec|spec-pr|spec-contracts) ;;
    *) 8>&- mustmatch test "${MD_PATHS[@]}" --lang bash "${timeout_args[@]}"; return ;;
  esac

  local worker_count="$SPEC_WORKER_COUNT"

  mkdir -p "$ROOT/.cache"
  PARALLEL_SPEC_OUTPUT_DIR="$(mktemp -d "$ROOT/.cache/spec-parallel.XXXXXX")"

  local path log_path pid page_index=0 wait_index batch_size=0
  local -a batch_paths=() batch_logs=()
  local batch_failed=0 exit_status=0
  for path in "${MD_PATHS[@]}"; do
    log_path="$PARALLEL_SPEC_OUTPUT_DIR/$page_index.log"
    # Monitor mode gives each background page its own process group. That lets
    # interruption reap Mustmatch and any command it currently owns.
    set -m
    (
      if ctgov_page_consumes_request_log "$path"; then
        prepare_ctgov_page_request_log
      fi
      8>&- exec mustmatch test "$path" --lang bash "${timeout_args[@]}"
    ) >"$log_path" 2>&1 &
    pid=$!
    set +m
    PARALLEL_SPEC_PIDS+=("$pid")
    batch_paths+=("$path")
    batch_logs+=("$log_path")
    page_index=$((page_index + 1))
    batch_size=$((batch_size + 1))

    if ((batch_size < worker_count && page_index < ${#MD_PATHS[@]})); then
      continue
    fi

    batch_failed=0
    for wait_index in "${!PARALLEL_SPEC_PIDS[@]}"; do
      if wait "${PARALLEL_SPEC_PIDS[$wait_index]}"; then
        exit_status=0
      else
        exit_status=$?
        batch_failed=1
      fi
      cat "${batch_logs[$wait_index]}"
      if ((exit_status != 0)); then
        printf 'spec page failed: %s (exit %s)\n' \
          "${batch_paths[$wait_index]}" "$exit_status" >&2
      fi
    done
    PARALLEL_SPEC_PIDS=()
    batch_paths=()
    batch_logs=()
    batch_size=0
    ((batch_failed == 0)) || return 1
  done

  rm -r "$PARALLEL_SPEC_OUTPUT_DIR"
  PARALLEL_SPEC_OUTPUT_DIR=""
}

run_python_contracts() {
  if ((${#PY_PATHS[@]})); then
    8>&- uv run --with pytest --no-project pytest "${PY_PATHS[@]}" -v
  fi
}

prepare_spec_artifacts() {
  # Small copied workspaces in runner lifecycle tests intentionally contain no
  # Cargo project. A real BioMCP checkout always has this manifest.
  [[ -f "$ROOT/Cargo.toml" ]] || return 0

  local env_file="$ROOT/.cache/spec-artifacts.env"
  mkdir -p "$ROOT/.cache"
  : "${ROUTINE_CARGO_FEATURES:?run specification modes through a Make target that declares the routine Cargo features}"
  local -a routine_cargo_features=()
  read -r -a routine_cargo_features <<< "$ROUTINE_CARGO_FEATURES"
  ((${#routine_cargo_features[@]})) || {
    echo "ROUTINE_CARGO_FEATURES must select the routine Cargo graph" >&2
    return 1
  }
  local -a arguments=(
    --mode "$mode"
    --profile "${SPEC_PROFILE:-spec}"
    --output "$env_file"
  )
  local cargo_feature_arg
  for cargo_feature_arg in "${routine_cargo_features[@]}"; do
    arguments+=("--cargo-feature-arg=$cargo_feature_arg")
  done
  if [[ -n "${BIOMCP_FEATURE_ON_BIN:-}" ]]; then
    arguments+=(--feature-on-bin "$BIOMCP_FEATURE_ON_BIN")
  fi
  if [[ "${BIOMCP_SPEC_ARTIFACTS_PREPARED:-0}" != 1 ]]; then
    python3 scripts/prepare-spec-artifacts.py "${arguments[@]}"
  fi
  [[ -s "$env_file" ]] || {
    echo "spec preparation did not create $env_file" >&2
    return 1
  }
  unset BIOMCP_SPEC_FEATURE_OFF_BIN BIOMCP_SPEC_FEATURE_ON_BIN
  unset BIOMCP_SPEC_MCP_EXAMPLE_BIN BIOMCP_SPEC_CARGO_TREE
  unset BIOMCP_SPEC_CARGO_METADATA BIOMCP_SPEC_TEST_LIB
  unset BIOMCP_SPEC_TEST_ARTICLE_CLI_TESTS_STRUCTURE
  unset BIOMCP_SPEC_TEST_BENCHMARK_CLI_STRUCTURE
  unset BIOMCP_SPEC_TEST_CLI_LINE_CAP_ABSORPTION
  unset BIOMCP_SPEC_TEST_HEALTH_CLI_STRUCTURE BIOMCP_SPEC_TEST_LIST_CLI_STRUCTURE
  unset BIOMCP_SPEC_TEST_SKILL_CLI_STRUCTURE
  # shellcheck source=/dev/null
  . "$env_file"
  local -a required_paths=()
  case "$mode" in
    spec|spec-pr|spec-contracts)
      required_paths=(
        "$BIOMCP_SPEC_FEATURE_OFF_BIN"
        "$BIOMCP_SPEC_MCP_EXAMPLE_BIN"
        "$BIOMCP_SPEC_CARGO_TREE"
        "$BIOMCP_SPEC_CARGO_METADATA"
      )
      ;;
    verify)
      required_paths=(
        "$BIOMCP_SPEC_FEATURE_OFF_BIN"
        "$BIOMCP_SPEC_FEATURE_ON_BIN"
        "$BIOMCP_SPEC_TEST_LIB"
        "$BIOMCP_SPEC_TEST_ARTICLE_CLI_TESTS_STRUCTURE"
        "$BIOMCP_SPEC_TEST_BENCHMARK_CLI_STRUCTURE"
        "$BIOMCP_SPEC_TEST_CLI_LINE_CAP_ABSORPTION"
        "$BIOMCP_SPEC_TEST_HEALTH_CLI_STRUCTURE"
        "$BIOMCP_SPEC_TEST_LIST_CLI_STRUCTURE"
        "$BIOMCP_SPEC_TEST_SKILL_CLI_STRUCTURE"
      )
      ;;
    verify-nih-reporter)
      required_paths=("$BIOMCP_SPEC_FEATURE_ON_BIN" "$BIOMCP_SPEC_TEST_LIB")
      ;;
  esac
  local required
  for required in "${required_paths[@]}"; do
    [[ -s "$required" ]] || {
      echo "prepared spec artifact is missing or empty: $required" >&2
      return 1
    }
  done
  export BIOMCP_BIN BIOMCP_SPEC_FEATURE_OFF_BIN BIOMCP_SPEC_FEATURE_ON_BIN
  export BIOMCP_SPEC_MCP_EXAMPLE_BIN BIOMCP_SPEC_CARGO_TREE
  export BIOMCP_SPEC_CARGO_METADATA BIOMCP_SPEC_ARTIFACT_MODE
  export BIOMCP_SPEC_TEST_LIB BIOMCP_SPEC_TEST_ARTICLE_CLI_TESTS_STRUCTURE
  export BIOMCP_SPEC_TEST_BENCHMARK_CLI_STRUCTURE
  export BIOMCP_SPEC_TEST_CLI_LINE_CAP_ABSORPTION
  export BIOMCP_SPEC_TEST_HEALTH_CLI_STRUCTURE BIOMCP_SPEC_TEST_LIST_CLI_STRUCTURE
  export BIOMCP_SPEC_TEST_SKILL_CLI_STRUCTURE
}

mode="${1:-}"
case "$mode" in
  prepare-spec)
    mode=spec
    prepare_spec_artifacts
    exit 0
    ;;
  spec|spec-pr)
    timeout_args=(--timeout 180)
    paths=("${SPEC_ROUTINE_PATHS[@]}")
    validate_routine_spec_workers
    mustmatch_path_dir="$(mustmatch_dir)"
    prepare_spec_artifacts
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
    run_vaers_fixture
    # The provider fixture owns the shared OpenFDA base in the combined suite;
    # the standalone VAERS fixture still exports its self-contained base.
    run_provider_contract_fixture
    run_variant_identity_fixture
    run_clingen_cspec_fixture
    run_cpic_fixture
    if paths_include_any spec/entity/protein.md; then
      run_protein_fixture
    fi
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
    validate_routine_spec_workers
    mustmatch_path_dir="$(mustmatch_dir)"
    prepare_spec_artifacts
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
    paths=("${SPEC_LIVE_PATHS[@]}")
    mustmatch_path_dir="$(mustmatch_dir)"
    BIOMCP_FEATURE_ON_BIN="${BIOMCP_FEATURE_ON_BIN:-${BIOMCP_BIN:-}}"
    prepare_spec_artifacts
    run_live_ddinter_root
    ;;
  verify-nih-reporter)
    timeout_args=(--timeout 180)
    paths=("${SPEC_NIH_REPORTER_LIVE_PATHS[@]}")
    mustmatch_path_dir="$(mustmatch_dir)"
    BIOMCP_FEATURE_ON_BIN="${BIOMCP_FEATURE_ON_BIN:-${BIOMCP_BIN:-}}"
    prepare_spec_artifacts
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

partition_paths "${paths[@]}"
run_article_markdown_specs
run_markdown_specs
run_section_outcome_specs
run_python_contracts
