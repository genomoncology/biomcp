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

write_record() {
  local workspace="$1" kind="$2" env_file="$3" root="$4" pid="$5" prefix="$6" owner_arg="$7"
  local canonical_workspace canonical_root token start
  canonical_workspace="$(canonical_dir "$workspace")"
  canonical_root="$(canonical_dir "$root")"
  token="${owner_arg#routine-fixture-owner:${kind}:}"
  token="${token%%:*}"
  [[ "$owner_arg" == "routine-fixture-owner:${kind}:${token}:${canonical_root}" ]] || return 1
  start="$(proc_start_identity "$pid")"
  {
    printf 'export %s_RECORD_VERSION=1\n' "$prefix"
    printf 'export %s_PID=%q\n' "$prefix" "$pid"
    printf 'export %s_SERVER_PID=%q\n' "$prefix" "$pid"
    printf 'export %s_PGID=%q\n' "$prefix" "$pid"
    printf 'export %s_ROOT=%q\n' "$prefix" "$canonical_root"
    printf 'export %s_PID_START_ID=%q\n' "$prefix" "$start"
    printf 'export %s_OWNER_WORKTREE=%q\n' "$prefix" "$canonical_workspace"
    printf 'export %s_OWNER_TOKEN=%q\n' "$prefix" "$token"
    printf 'export %s_OWNER_ARG=%q\n' "$prefix" "$owner_arg"
  } >>"$env_file"
  printf '%s\n' "$owner_arg"
}

record_value() {
  local env_file="$1" wanted="$2" line key value result=""
  while IFS= read -r line || [[ -n "$line" ]]; do
    [[ "$line" == "export $wanted="* ]] || continue
    key="${line%%=*}"
    value="${line#*=}"
    [[ "$key" == "export $wanted" && "$value" =~ ^[A-Za-z0-9_./:-]+$ ]] || return 1
    [[ -z "$result" ]] || return 1
    result="$value"
  done <"$env_file"
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
  local workspace="$1" kind="$2" env_file="$3" prefix="$4"
  [[ -f "$env_file" ]] || return 0
  local version pid pgid root start worktree token owner_arg actual_pgid actual_start cmdline
  version="$(record_value "$env_file" "${prefix}_RECORD_VERSION")" || { rm -f "$env_file"; return 0; }
  pid="$(record_value "$env_file" "${prefix}_PID")" || { rm -f "$env_file"; return 0; }
  pgid="$(record_value "$env_file" "${prefix}_PGID")" || { rm -f "$env_file"; return 0; }
  root="$(record_value "$env_file" "${prefix}_ROOT")" || { rm -f "$env_file"; return 0; }
  start="$(record_value "$env_file" "${prefix}_PID_START_ID")" || { rm -f "$env_file"; return 0; }
  worktree="$(record_value "$env_file" "${prefix}_OWNER_WORKTREE")" || { rm -f "$env_file"; return 0; }
  token="$(record_value "$env_file" "${prefix}_OWNER_TOKEN")" || { rm -f "$env_file"; return 0; }
  owner_arg="$(record_value "$env_file" "${prefix}_OWNER_ARG")" || { rm -f "$env_file"; return 0; }
  [[ "$version" == 1 && "$pid" =~ ^[1-9][0-9]*$ && "$pgid" == "$pid" ]] || { rm -f "$env_file"; return 0; }
  [[ "$owner_arg" == "routine-fixture-owner:${kind}:${token}:"* ]] || { rm -f "$env_file"; return 0; }
  [[ "$worktree" == "$(canonical_dir "$workspace")" ]] || { rm -f "$env_file"; return 0; }
  root_is_owned "$workspace" "$kind" "$root" || { rm -f "$env_file"; return 0; }
  [[ -r "/proc/$pid/stat" && -r "/proc/$pid/cmdline" ]] || { rm -f "$env_file"; return 0; }
  actual_pgid="$(ps -o pgid= -p "$pid" 2>/dev/null | tr -d ' ')" || { rm -f "$env_file"; return 0; }
  actual_start="$(proc_start_identity "$pid")" || { rm -f "$env_file"; return 0; }
  cmdline="$(tr '\0' ' ' <"/proc/$pid/cmdline")"
  [[ "$actual_pgid" == "$pgid" && "$actual_start" == "$start" && "$cmdline" == *"$owner_arg"* ]] || { rm -f "$env_file"; return 0; }
  kill -TERM -- "-$pgid" 2>/dev/null || true
  for _ in $(seq 1 50); do
    kill -0 -- "-$pgid" 2>/dev/null || break
    sleep 0.1
  done
  kill -KILL -- "-$pgid" 2>/dev/null || true
  rm -rf "$root"
  rm -f "$env_file"
}

case "${1:-}" in
  new-owner) shift; new_owner_arg "$@" ;;
  write) shift; write_record "$@" ;;
  cleanup) shift; cleanup_record "$@" ;;
  *) echo "usage: $0 {write|cleanup} ..." >&2; exit 2 ;;
esac
