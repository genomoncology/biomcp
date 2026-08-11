#!/usr/bin/env bash
# Argument adapter for the shared Python fixture lifecycle supervisor.
set -euo pipefail

_fixture_supervisor_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

_fixture_proc_start_identity() {
  local stat rest
  stat="$(<"/proc/$1/stat")" || return 1
  rest="${stat#*) }"
  local -a fields
  read -r -a fields <<<"$rest"
  printf '%s\n' "${fields[19]}"
}

prepare_fixture_supervisor_owner() {
  if [[ -v ROUTINE_FIXTURE_OWNER_PID || -v ROUTINE_FIXTURE_OWNER_START_ID ]]; then
    fixture_supervisor_owner_pid="${ROUTINE_FIXTURE_OWNER_PID:-}"
    fixture_supervisor_owner_start="${ROUTINE_FIXTURE_OWNER_START_ID:-}"
    [[ "$fixture_supervisor_owner_pid" =~ ^[1-9][0-9]*$ && "$fixture_supervisor_owner_start" =~ ^[0-9]+$ ]] || {
      echo "fixture supervisor: invalid routine owner identity" >&2
      return 2
    }
  else
    fixture_supervisor_owner_pid="$PPID"
    fixture_supervisor_owner_start="$(
      _fixture_proc_start_identity "$fixture_supervisor_owner_pid"
    )" || {
      echo "fixture supervisor: could not read standalone caller identity" >&2
      return 2
    }
  fi
  [[ "$(_fixture_proc_start_identity "$fixture_supervisor_owner_pid")" == "$fixture_supervisor_owner_start" ]] || {
    echo "fixture supervisor: routine owner identity does not match procfs" >&2
    return 2
  }
}

start_fixture_supervisor() {
  local parent="$1" root="$2" prefix="$3" pid_file="$4"
  shift 4
  if [[ -z "${fixture_supervisor_owner_pid:-}" || -z "${fixture_supervisor_owner_start:-}" ]]; then
    prepare_fixture_supervisor_owner
  fi
  exec 8>&-
  exec setsid python3 "$_fixture_supervisor_dir/fixture-supervisor.py" launch \
    "$fixture_supervisor_owner_pid" "$fixture_supervisor_owner_start" \
    "$parent" "$root" "$prefix" "$pid_file" -- "$@"
}

recover_disease_survival_orphans() {
  local parent="$1"
  python3 "$_fixture_supervisor_dir/fixture-supervisor.py" recover-disease \
    "$parent" "spec-disease-survival."
}

recover_provider_contract_orphans() {
  local parent="$1"
  python3 "$_fixture_supervisor_dir/fixture-supervisor.py" recover-disease \
    "$parent" "spec-provider-contract."
}
