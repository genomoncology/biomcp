#!/usr/bin/env bash
# Shared ownership records for runner-owned routine fixture process groups.
set -euo pipefail

canonical_dir() {
  realpath -e "$1"
}

proc_start_identity() {
  local stat rest
  stat="$(<"/proc/$1/stat")" || return 1
  rest="${stat#*) }"
  local -a fields
  read -r -a fields <<<"$rest"
  printf '%s\n' "${fields[19]}"
}

new_owner_arg() {
  local kind="$1" root="$2" token canonical_root
  canonical_root="$(canonical_dir "$root")"
  token="$(od -An -N16 -tx1 /dev/urandom | tr -d ' \n')"
  printf 'routine-fixture-owner:%s:%s:%s\n' "$kind" "$token" "$canonical_root"
}

ownership_record_file() {
  printf '%s/.cache/spec-%s-ownership\n' "$(canonical_dir "$1")" "$2"
}

write_record() {
  local workspace="$1" kind="$2" root="$3" pid="$4" prefix="$5" owner_arg="$6"
  local canonical_workspace canonical_root token start record_file
  canonical_workspace="$(canonical_dir "$workspace")"
  canonical_root="$(canonical_dir "$root")"
  token="${owner_arg#routine-fixture-owner:${kind}:}"
  token="${token%%:*}"
  [[ "$owner_arg" == "routine-fixture-owner:${kind}:${token}:${canonical_root}" ]] || return 1
  start="$(proc_start_identity "$pid")"
  record_file="$(ownership_record_file "$workspace" "$kind")"
  {
    printf '%s_RECORD_VERSION=1\n' "$prefix"
    printf '%s_PID=%s\n' "$prefix" "$pid"
    printf '%s_SERVER_PID=%s\n' "$prefix" "$pid"
    printf '%s_PGID=%s\n' "$prefix" "$pid"
    printf '%s_ROOT=%s\n' "$prefix" "$canonical_root"
    printf '%s_PID_START_ID=%s\n' "$prefix" "$start"
    printf '%s_OWNER_WORKTREE=%s\n' "$prefix" "$canonical_workspace"
    printf '%s_OWNER_TOKEN=%s\n' "$prefix" "$token"
    printf '%s_OWNER_ARG=%s\n' "$prefix" "$owner_arg"
  } >"$record_file"
  printf '%s\n' "$owner_arg"
}

record_has_only_known_fields() {
  local record_file="$1" prefix="$2" line
  while IFS= read -r line || [[ -n "$line" ]]; do
    case "$line" in
      "${prefix}_RECORD_VERSION="*|"${prefix}_PID="*|"${prefix}_SERVER_PID="*|\
      "${prefix}_PGID="*|"${prefix}_ROOT="*|"${prefix}_PID_START_ID="*|\
      "${prefix}_OWNER_WORKTREE="*|"${prefix}_OWNER_TOKEN="*|"${prefix}_OWNER_ARG="*) ;;
      *) return 1 ;;
    esac
  done <"$record_file"
}

record_value() {
  local record_file="$1" wanted="$2" line key value result=""
  while IFS= read -r line || [[ -n "$line" ]]; do
    [[ "$line" == "$wanted="* ]] || continue
    key="${line%%=*}"
    value="${line#*=}"
    [[ "$key" == "$wanted" ]] || return 1
    [[ -z "$result" ]] || return 1
    result="$value"
  done <"$record_file"
  [[ -n "$result" ]] || return 1
  printf '%s\n' "$result"
}

root_is_owned() {
  local workspace="$1" kind="$2" root="$3" cache canonical_workspace canonical_root
  canonical_workspace="$(canonical_dir "$workspace")" || return 1
  canonical_root="$(canonical_dir "$root")" || return 1
  cache="$canonical_workspace/.cache"
  [[ "$(dirname "$canonical_root")" == "$cache" && "$(basename "$canonical_root")" == "spec-$kind".* ]]
}

cleanup_record() {
  local workspace="$1" kind="$2" prefix="$3" record_file
  record_file="$(ownership_record_file "$workspace" "$kind")"
  [[ -f "$record_file" ]] || return 0
  local version pid pgid root start worktree token owner_arg actual_pgid actual_start cmdline
  record_has_only_known_fields "$record_file" "$prefix" || { rm -f "$record_file"; return 0; }
  version="$(record_value "$record_file" "${prefix}_RECORD_VERSION")" || { rm -f "$record_file"; return 0; }
  pid="$(record_value "$record_file" "${prefix}_PID")" || { rm -f "$record_file"; return 0; }
  pgid="$(record_value "$record_file" "${prefix}_PGID")" || { rm -f "$record_file"; return 0; }
  root="$(record_value "$record_file" "${prefix}_ROOT")" || { rm -f "$record_file"; return 0; }
  start="$(record_value "$record_file" "${prefix}_PID_START_ID")" || { rm -f "$record_file"; return 0; }
  worktree="$(record_value "$record_file" "${prefix}_OWNER_WORKTREE")" || { rm -f "$record_file"; return 0; }
  token="$(record_value "$record_file" "${prefix}_OWNER_TOKEN")" || { rm -f "$record_file"; return 0; }
  owner_arg="$(record_value "$record_file" "${prefix}_OWNER_ARG")" || { rm -f "$record_file"; return 0; }
  [[ "$version" == 1 && "$pid" =~ ^[1-9][0-9]*$ && "$pgid" == "$pid" ]] || { rm -f "$record_file"; return 0; }
  [[ "$owner_arg" == "routine-fixture-owner:${kind}:${token}:${root}" ]] || { rm -f "$record_file"; return 0; }
  [[ "$worktree" == "$(canonical_dir "$workspace")" ]] || { rm -f "$record_file"; return 0; }
  root_is_owned "$workspace" "$kind" "$root" || { rm -f "$record_file"; return 0; }
  [[ -r "/proc/$pid/stat" && -r "/proc/$pid/cmdline" ]] || { rm -f "$record_file"; return 0; }
  actual_pgid="$(ps -o pgid= -p "$pid" 2>/dev/null | tr -d ' ')" || { rm -f "$record_file"; return 0; }
  actual_start="$(proc_start_identity "$pid")" || { rm -f "$record_file"; return 0; }
  cmdline="$(tr '\0' ' ' <"/proc/$pid/cmdline")"
  [[ "$actual_pgid" == "$pgid" && "$actual_start" == "$start" && "$cmdline" == *"$owner_arg"* ]] || { rm -f "$record_file"; return 0; }
  if [[ -n "${ROUTINE_FIXTURE_LOCK_PATH:-}" ]] && \
    { [[ ! -e "/proc/$pid/fd/8" ]] || [[ ! "/proc/$pid/fd/8" -ef "$ROUTINE_FIXTURE_LOCK_PATH" ]]; }; then
    return 0
  fi
  kill -TERM -- "-$pgid" 2>/dev/null || true
  for _ in $(seq 1 50); do
    kill -0 -- "-$pgid" 2>/dev/null || break
    [[ "$(ps -o stat= -p "$pid" 2>/dev/null | tr -d ' ')" == Z* ]] && break
    sleep 0.1
  done
  kill -KILL -- "-$pgid" 2>/dev/null || true
  rm -rf "$root"
  rm -f "$record_file"
}

case "${1:-}" in
  new-owner) shift; new_owner_arg "$@" ;;
  write) shift; write_record "$@" ;;
  cleanup) shift; cleanup_record "$@" ;;
  *) echo "usage: $0 {write|cleanup} ..." >&2; exit 2 ;;
esac
