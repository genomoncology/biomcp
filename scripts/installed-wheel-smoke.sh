#!/usr/bin/env bash
set -euo pipefail

# Run the installed artifact without the caller's configuration or credentials.
BIOMCP="$1"
expected="$2"
if [ "${BIOMCP_WHEEL_SMOKE_ISOLATED:-}" != 1 ]; then
  smoke_root="$(mktemp -d)"
  trap 'rm -rf "$smoke_root"' EXIT
  mkdir "$smoke_root/home"
  smoke_home="$smoke_root/home"
  if command -v cygpath >/dev/null 2>&1; then
    smoke_home="$(cygpath -w "$smoke_home")"
  fi
  env -i PATH="$PATH" SYSTEMROOT="${SYSTEMROOT:-}" WINDIR="${WINDIR:-}" \
    HOME="$smoke_home" USERPROFILE="$smoke_home" \
    LOCALAPPDATA="$smoke_home/AppData/Local" APPDATA="$smoke_home/AppData/Roaming" \
    BIOMCP_WHEEL_SMOKE_ISOLATED=1 \
    BIOMCP_DISABLE_SOURCES=commercial-restricted,kegg,semantic-scholar \
    bash --noprofile --norc "$0" "$BIOMCP" "$expected"
  exit
fi

printed="$("$BIOMCP" --version 2>&1)" || {
  printf '%s\n' "$printed" >&2
  exit 1
}
printf '%s\n' "$printed"
if [ "$(printf '%s' "$printed" | tr -d '\r\n')" != "biomcp $expected" ]; then
  echo "wheel --version must print exactly biomcp $expected, got: $printed" >&2
  exit 1
fi
stats="$("$BIOMCP" cache stats --json 2>&1)" || {
  printf '%s\n' "$stats" >&2
  echo "installed wheel cache stats failed" >&2
  exit 1
}
case "$stats" in
  "{"*) ;;
  *) echo "cache stats must print a JSON object, got: $stats" >&2; exit 1 ;;
esac


smoke_stderr="$(mktemp)"
trap 'rm -f "$smoke_stderr"' EXIT

require_exit() {
  sleep 1
  expected="$1"; shift
  status=0
  output="$("$BIOMCP" "$@" 2>&1)" || status=$?
  printf '%s\n' "$output" | sed -n '1,20p'
  if [ "$status" -eq 101 ] || [ "$status" -ge 128 ]; then
    printf '%s\n' "$output" >&2
    echo "panic or crash (exit $status) from: biomcp $*" >&2
    return 1
  fi
  if printf '%s\n' "$output" | grep -q "has overflowed its stack"; then
    printf '%s\n' "$output" >&2
    echo "stack overflow from: biomcp $*" >&2
    return 1
  fi
  if [ "$status" -ne "$expected" ]; then
    printf '%s\n' "$output" >&2
    echo "expected exit $expected, got $status from: biomcp $*" >&2
    return 1
  fi
}

require_json() {
  sleep 1
  status=0
  output="$("$BIOMCP" "$@" 2>"$smoke_stderr")" || status=$?
  cat "$smoke_stderr" >&2
  if [ "$status" -ne 0 ]; then
    printf '%s\n' "$output" >&2
    echo "json mode must exit 0 from: biomcp $* (exit $status)" >&2
    return 1
  fi
  case "$output" in
    "{"*) ;;
    *) echo "json mode must print an object from: biomcp $*" >&2; return 1 ;;
  esac
  if printf '%s\n' "$output" | grep -q "skill asset"; then
    printf '%s\n' "$output" >&2
    echo "skill asset error from: biomcp $*" >&2
    return 1
  fi
}

require_ddinter_disabled() {
  require_exit 1 "$@" || return 1
  if ! printf '%s\n' "$output" | grep -Fq "DDInter is disabled by the commercial-restricted group"; then
    printf '%s\n' "$output" >&2
    echo "expected the commercial-restricted DDInter disable message from: biomcp $*" >&2
    return 1
  fi
}

require_exit 0 search trial --condition diabetes --limit 1
require_exit 0 search trial --criteria "anti-PD-1 therapy" --limit 3
# This public smoke excludes restricted sources. Live DDInter
# sync and interaction data behavior remains unverified.
require_ddinter_disabled ddinter sync
require_ddinter_disabled drug interactions apixaban
require_exit 0 drug trials imatinib

sleep 1
status=0
output="$("$BIOMCP" drug adverse-events qwertyzzznonexistent999 2>&1)" || status=$?
printf '%s\n' "$output" | sed -n '1,20p'
if [ "$status" -ne 0 ] || ! printf '%s\n' "$output" | grep -q "Drug not found in FAERS"; then
  printf '%s\n' "$output" >&2
  echo "expected a successful FAERS not-found response" >&2
  exit 1
fi

require_json get drug --region us aspirin regulatory -j
require_json search trial --condition diabetes --limit 1 -j

# GitHub #287: the 0.9.0 debug-profile wheel embedded no
# assets, so skill, chart, and --json ladders failed on
# every PyPI install. Exercise those paths positively:
# each must succeed AND name real content, not an empty
# or missing-asset answer.
status=0
output="$("$BIOMCP" skill list 2>&1)" || status=$?
printf '%s\n' "$output" | sed -n '1,20p'
if [ "$status" -ne 0 ] || printf '%s\n' "$output" | grep -q "No skills found"; then
  echo "skill list must name skills (exit $status): #287 regression: $output" >&2
  exit 1
fi
# Ticket 1266: exit codes alone let the 0.9.0 debug-wheel
# failure through (errors exited 0), so assert known
# content from the embedded assets too — no network.
status=0
output="$("$BIOMCP" skill article-follow-up 2>&1)" || status=$?
printf '%s\n' "$output" | sed -n '1,20p'
if [ "$status" -ne 0 ] || ! printf '%s\n' "$output" | grep -q "Article follow-up via citations and recommendations"; then
  echo "skill article-follow-up must show its content (exit $status): #287 regression: $output" >&2
  exit 1
fi
status=0
output="$("$BIOMCP" chart bar 2>&1)" || status=$?
printf '%s\n' "$output" | sed -n '1,20p'
if [ "$status" -ne 0 ] || ! printf '%s\n' "$output" | grep -q "Bar Chart"; then
  echo "chart bar must show the Bar Chart heading (exit $status): #287 regression: $output" >&2
  exit 1
fi
require_json --json get article 32794606
