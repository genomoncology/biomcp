#!/usr/bin/env bash
# Install the pinned cargo-nextest release tarball with a checksum
# verify (2026-09-29 review: curl | tar without a checksum is an
# unverified supply-chain step). The version and the SHA-256 come
# from the workflow environment or the shared CI declaration; the
# checksum was taken from the release artifact itself.
set -euo pipefail

if [[ -z "${CARGO_NEXTEST_VERSION:-}" && -z "${CARGO_NEXTEST_SHA256:-}" ]]; then
  root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
  version="$(sed -n 's/^  CARGO_NEXTEST_VERSION: \([0-9.]*\)$/\1/p' "$root/.github/workflows/ci.yml")"
  sha="$(sed -n 's/^  CARGO_NEXTEST_SHA256: \([0-9a-f]*\)$/\1/p' "$root/.github/workflows/ci.yml")"
else
  version="${CARGO_NEXTEST_VERSION:?CARGO_NEXTEST_VERSION must be set}"
  sha="${CARGO_NEXTEST_SHA256:?CARGO_NEXTEST_SHA256 must be set}"
fi
[[ "$version" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ && "$sha" =~ ^[0-9a-f]{64}$ ]] || {
  echo "nextest requires one valid version and checksum from the CI declaration" >&2
  exit 1
}
url="https://github.com/nextest-rs/nextest/releases/download/cargo-nextest-${version}/cargo-nextest-${version}-x86_64-unknown-linux-musl.tar.gz"

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT
curl -fsSL -o "$tmp/nextest.tgz" "$url"
echo "${sha}  ${tmp}/nextest.tgz" | sha256sum --check --status
tar xzf "$tmp/nextest.tgz" -C "${HOME}/.cargo/bin"
