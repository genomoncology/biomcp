#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

# The routine spec pages run offline against one replay server
# (tools/replay-server) over the recorded replies in testdata/replay, and
# their JSON and request-log checks live in the golden cases under
# tests/golden/spec. Provider-backed pages whose recordings are not authored
# yet stay out of the lists below; the ticket that converts a page re-lists
# it.
SPEC_ROUTINE_PATHS=(
  spec/surface/cli-contract-ratchet.md
  spec/surface/build-profile.md
  spec/surface/skills.md
  spec/entity/trial-numeric-filters.md
  spec/surface/trial-retirement.md
  # Replays its own frozen capture through spec/fixtures/run-clingen-erepo-
  # fixture.sh, not the replay server (2127 ruling).
  spec/entity/clingen-erepo.md
  # Replays its own frozen corpus through spec/fixtures/run-variant-articles-
  # corpus-canary.sh, not the replay server (2127 ruling).
  spec/entity/variant-articles-corpus.md
  tests/golden/spec
)

SPEC_STATIC_PATHS=(
  spec/surface/docker-image.md
  spec/surface/homebrew.md
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

partition_paths() {
  MD_PATHS=()
  PY_PATHS=()
  local path
  for path in "$@"; do
    case "$path" in
      *.md) MD_PATHS+=("$path") ;;
      *.py|tests/golden/spec) PY_PATHS+=("$path") ;;
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

FAILED_SPEC_ENTRIES=()

record_spec_failure() {
  FAILED_SPEC_ENTRIES+=("$1")
}

# Run a stage to completion and remember its failure instead of aborting the
# run. A stage that already named the pages it lost keeps that detail.
run_spec_stage() {
  local stage="$1" before="${#FAILED_SPEC_ENTRIES[@]}"
  if ! "$stage"; then
    ((${#FAILED_SPEC_ENTRIES[@]} > before)) || record_spec_failure "stage $stage"
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
  # Bash 3.2 (macOS) treats "${arr[@]}" on an empty array as an unbound
  # variable under set -u; the ${arr[@]+...} expansion stays safe everywhere.
  for cleanup_function in ${CLEANUP_FUNCTIONS[@]+"${CLEANUP_FUNCTIONS[@]}"}; do
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

REPLAY_SERVER_PID=""

stop_replay_server() {
  if [[ -n "$REPLAY_SERVER_PID" ]]; then
    kill "$REPLAY_SERVER_PID" 2>/dev/null || true
    wait "$REPLAY_SERVER_PID" 2>/dev/null || true
    REPLAY_SERVER_PID=""
  fi
  if [[ -n "${REPLAY_GREETING_FD:-}" ]]; then
    eval "exec ${REPLAY_GREETING_FD}>&-"
    REPLAY_GREETING_FD=""
  fi
}

# One replay server stands in for every fixture script the routine lane used
# to start: every provider base override points at it, it serves the recorded
# replies under testdata/replay, and the golden cases in tests/golden/spec
# assert on its request log. Copied workspaces in runner contract tests may
# carry no replay server; without one there is nothing to start and the
# pages name their own failure.
REPLAY_GREETING_FD=""

run_replay_server() {
  [[ -x tools/replay-server ]] || {
    echo "tools/replay-server is missing; the routine spec lane needs the replay server" >&2
    return 1
  }
  local env_file="$ROOT/.cache/spec-replay-env"
  local request_log="$ROOT/.cache/spec-replay-request.log"
  mkdir -p "$ROOT/.cache"
  rm -f "$env_file" "$request_log"
  # The server greets with one READY line carrying its origin. Blocking on
  # that line observes readiness without a timed startup poll, and a server
  # that dies before serving closes the pipe; the read is bounded so a dead
  # server fails the lane loudly instead of hanging it.
  # The greeting pipe lives on fd 3: dynamic `{var}<` allocation is Bash
  # 4.1+ and the macOS gate host runs the bundled Bash 3.2.
  local greeting origin
  exec 3< <(
    python3 tools/replay-server \
      --root "$ROOT" \
      --env-file "$env_file" \
      --request-log "$request_log"
  )
  REPLAY_GREETING_FD=3
  REPLAY_SERVER_PID=$!
  if ! IFS= read -r -t 60 -u "$REPLAY_GREETING_FD" greeting \
    || [[ "$greeting" != READY* ]]; then
    echo "replay server did not greet with READY: ${greeting:-no greeting}" >&2
    stop_replay_server
    return 1
  fi
  origin="${greeting#READY }"
  # shellcheck source=/dev/null
  . "$env_file"
  # Pages count their own provider traffic through the shared request log, so
  # the path every page greps is one exported variable.
  export BIOMCP_REPLAY_REQUEST_LOG="$request_log"
  # Every provider shares the replay origin, so one unpaced-origin signal
  # removes provider pacing, and recorded failures answer instantly, so the
  # retry back-off sleeps are turned off for the whole lane. The lane's
  # commands get their own empty cache root so no page reads or cleans
  # another lane's cache under the offline sandbox.
  export BIOMCP_TEST_UNPACED_ORIGIN="$origin"
  export BIOMCP_TEST_RETRY_SLEEP_OFF=1
  export BIOMCP_CACHE_DIR="$ROOT/.cache/spec-replay-http-cache"
  rm -rf "$BIOMCP_CACHE_DIR"
  mkdir -p "$BIOMCP_CACHE_DIR"
  register_cleanup stop_replay_server
}

run_markdown_specs() {
  ((${#MD_PATHS[@]})) || return 0
  printf '=== spec pages: %s ===\n' "${MD_PATHS[*]}"
  mustmatch test "${MD_PATHS[@]}" --lang bash --timeout 180
}

run_python_contracts() {
  if ((${#PY_PATHS[@]})); then
    uv run --with pytest --no-project pytest "${PY_PATHS[@]}" -v
  fi
}

prepare_spec_artifacts() {
  # Small copied workspaces in runner contract tests intentionally contain no
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
    --profile "${SPEC_PROFILE:-dev}"
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
  export BIOMCP_SPEC_TEST_LIB BIOMCP_SPEC_TEST_CLI_LINE_CAP_ABSORPTION
  export BIOMCP_SPEC_TEST_HEALTH_CLI_STRUCTURE BIOMCP_SPEC_TEST_LIST_CLI_STRUCTURE
  export BIOMCP_SPEC_TEST_SKILL_CLI_STRUCTURE
}

run_live_ddinter_root() {
  export BIOMCP_DDINTER_DIR="$ROOT/.cache/verify-ddinter-live"
  rm -rf "$BIOMCP_DDINTER_DIR"
}

mode="${1:-}"
case "$mode" in
  prepare-spec)
    mode=spec
    prepare_spec_artifacts
    exit 0
    ;;
  spec|spec-pr)
    paths=("${SPEC_ROUTINE_PATHS[@]}")
    mustmatch_path_dir="$(mustmatch_dir)"
    prepare_spec_artifacts
    run_replay_server
    ;;
  spec-contracts)
    # The deterministic subset contract lanes used to prove now matches the
    # routine list: both run offline against the replay server.
    paths=("${SPEC_ROUTINE_PATHS[@]}")
    mustmatch_path_dir="$(mustmatch_dir)"
    prepare_spec_artifacts
    run_replay_server
    ;;
  spec-static)
    paths=("${SPEC_STATIC_PATHS[@]}")
    mustmatch_path_dir="$(mustmatch_dir)"
    ;;
  verify)
    paths=("${SPEC_LIVE_PATHS[@]}")
    mustmatch_path_dir="$(mustmatch_dir)"
    BIOMCP_FEATURE_ON_BIN="${BIOMCP_FEATURE_ON_BIN:-${BIOMCP_BIN:-}}"
    prepare_spec_artifacts
    run_live_ddinter_root
    ;;
  verify-nih-reporter)
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

if [[ "$mode" == "spec-static" ]]; then
  export PATH="$mustmatch_path_dir:$PATH"
else
  case "$mode" in
    verify) default_biomcp_bin="$ROOT/target/release/biomcp" ;;
    verify-nih-reporter) default_biomcp_bin="$ROOT/target/release/biomcp" ;;
    *) default_biomcp_bin="$ROOT/target/debug/biomcp" ;;
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
run_spec_stage run_markdown_specs
run_spec_stage run_python_contracts

if ((${#FAILED_SPEC_ENTRIES[@]})); then
  printf 'spec run failed (%s):\n' "${#FAILED_SPEC_ENTRIES[@]}" >&2
  for failed_entry in "${FAILED_SPEC_ENTRIES[@]}"; do
    printf '  %s\n' "$failed_entry" >&2
  done
  exit 1
fi
