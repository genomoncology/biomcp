#!/usr/bin/env bash
set -euo pipefail

version=2.1.2
integrity='sha512-goRbBC8ySo7SWb7tRzr+tL6FxDc4JPTRCdgfD2omba7freofvjq5rom1lBnYHZHo6Mizs1jAHJeN53aZbDoy8A=='
url="https://registry.npmjs.org/@anthropic-ai/mcpb/-/mcpb-${version}.tgz"
temporary="$(mktemp -d)"
trap 'rm -rf "$temporary"' EXIT
curl --proto '=https' --tlsv1.2 --fail --silent --show-error --location "$url" --output "$temporary/mcpb.tgz"
actual="sha512-$(openssl dgst -sha512 -binary "$temporary/mcpb.tgz" | base64 | tr -d '\n')"
[[ "$actual" == "$integrity" ]] || {
  printf 'MCPB tool integrity mismatch\n' >&2
  exit 1
}
npm install --global --ignore-scripts "$temporary/mcpb.tgz"
[[ "$(mcpb --version)" == "$version" ]]
