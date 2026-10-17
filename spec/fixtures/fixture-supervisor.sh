#!/usr/bin/env bash
# Argument adapter for the shared Python fixture lifecycle supervisor.
set -euo pipefail

# Written for the macOS-bundled Bash 3.2 and both gate platforms: no
# `[[ -v ]]`, no GNU-only tools. Process data comes from procfs where it
# exists and from `ps` everywhere else.

_fixture_supervisor_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

_fixture_proc_start_identity() {
  # One definition of a process's start identity, shared by the routine spec
  # runner and the ownership records. Linux reads procfs jiffies; elsewhere
  # `ps` reports the kernel start time, coarse to the second. The value is
  # only ever compared for equality on the host that produced it.
  if [[ -r "/proc/$1/stat" ]]; then
    local stat rest
    stat="$(<"/proc/$1/stat")" || return 1
    rest="${stat#*) }"
    local -a fields
    read -r -a fields <<<"$rest"
    printf '%s\n' "${fields[19]}"
  else
    local identity
    identity="$(ps -o lstart= -p "$1" 2>/dev/null)" || return 1
    read -r identity <<<"$identity"
    [[ -n "$identity" ]] || return 1
    printf '%s\n' "$identity"
  fi
}

canonical_dir() {
  # realpath -e is GNU-only; cd plus pwd -P resolves the same existing
  # directory on every platform the gates run on.
  local dir
  dir="$(cd "$1" && pwd -P)" || return 1
  printf '%s\n' "$dir"
}

acquire_fixture_lock() {
  # flock(1) is Linux-only. The fallback takes the same advisory lock on the
  # same open file description with fcntl.flock in a Python child that
  # inherits the descriptor, so every platform serializes identically.
  local fd="$1" wait_seconds="${2:-}"
  if command -v flock >/dev/null 2>&1; then
    if [[ -n "$wait_seconds" ]]; then
      flock -w "$wait_seconds" "$fd"
    else
      flock "$fd"
    fi
    return
  fi
  python3 - "$fd" "$wait_seconds" <<'PY'
import errno
import fcntl
import sys
import time

fd = int(sys.argv[1])
deadline = (
    None
    if not sys.argv[2]
    else time.monotonic() + float(sys.argv[2])
)
while True:
    try:
        fcntl.flock(fd, fcntl.LOCK_EX | fcntl.LOCK_NB)
        raise SystemExit(0)
    except OSError as error:
        if error.errno not in (errno.EACCES, errno.EAGAIN):
            raise SystemExit(1)
    if deadline is not None and time.monotonic() >= deadline:
        raise SystemExit(1)
    time.sleep(0.2)
PY
}

run_with_timeout() {
  # GNU timeout(1) is absent on macOS; the fallback keeps its 124 exit code
  # so pages assert the same deadline verdict on every gate host.
  local seconds="$1"
  shift
  if command -v timeout >/dev/null 2>&1; then
    timeout "$seconds" "$@"
    return
  fi
  python3 - "$seconds" -- "$@" <<'PY'
import subprocess
import sys

try:
    completed = subprocess.run(sys.argv[3:], timeout=float(sys.argv[1]))
except subprocess.TimeoutExpired:
    print(f"command timed out after {sys.argv[1]} seconds", file=sys.stderr)
    raise SystemExit(124)
raise SystemExit(completed.returncode)
PY
}

prepare_fixture_supervisor_owner() {
  if [[ "${ROUTINE_FIXTURE_OWNER_PID+set}" == set || "${ROUTINE_FIXTURE_OWNER_START_ID+set}" == set ]]; then
    fixture_supervisor_owner_pid="${ROUTINE_FIXTURE_OWNER_PID:-}"
    fixture_supervisor_owner_start="${ROUTINE_FIXTURE_OWNER_START_ID:-}"
    [[ "$fixture_supervisor_owner_pid" =~ ^[1-9][0-9]*$ && -n "${fixture_supervisor_owner_start//[[:space:]]/}" ]] || {
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
    echo "fixture supervisor: routine owner identity does not match the running process" >&2
    return 2
  }
}

prepare_fixture_supervisor_current_process() {
  fixture_supervisor_owner_pid="$$"
  fixture_supervisor_owner_start="$(_fixture_proc_start_identity "$$")" || {
    echo "fixture supervisor: could not read wrapper identity" >&2
    return 2
  }
}

start_fixture_supervisor() {
  local kind="$1" parent="$2" root="$3" prefix="$4" pid_file="$5"
  shift 5
  if [[ -z "${fixture_supervisor_owner_pid:-}" || -z "${fixture_supervisor_owner_start:-}" ]]; then
    prepare_fixture_supervisor_owner
  fi
  exec 8>&-
  # The supervisor moves itself into a new session at launch, so no platform
  # needs a setsid(1) binary and the detached lifecycle is defined once.
  exec python3 "$_fixture_supervisor_dir/fixture-supervisor.py" launch \
    "$fixture_supervisor_owner_pid" "$fixture_supervisor_owner_start" \
    "$kind" "$parent" "$root" "$prefix" "$pid_file" -- "$@"
}

recover_fixture_orphans() {
  local parent="$1" kind="$2" prefix="$3"
  python3 "$_fixture_supervisor_dir/fixture-supervisor.py" recover \
    "$parent" "$kind" "$prefix"
}

recover_disease_survival_orphans() {
  recover_fixture_orphans "$1" "disease-survival" "spec-disease-survival."
}

recover_provider_contract_orphans() {
  recover_fixture_orphans "$1" "provider-contract" "spec-provider-contract."
}
