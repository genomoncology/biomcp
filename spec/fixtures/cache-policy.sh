#!/usr/bin/env bash
# Shared fixture cache policy. Source this file before selecting a cache root.
set_fixture_cache_root() {
  export BIOMCP_CACHE_DIR="$1"
  export BIOMCP_CACHE_MIN_DISK_FREE=1B
}

print_fixture_cache_exports() {
  printf 'export BIOMCP_CACHE_DIR=%q\n' "$1"
  printf 'export BIOMCP_CACHE_MIN_DISK_FREE=%q\n' 1B
}
