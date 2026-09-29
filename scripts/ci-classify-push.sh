#!/usr/bin/env bash
# Classify a push as docs-only by the files it changed across the
# whole push (2026-09-29 review): the commit message decides
# nothing, because a code push whose last commit says "docs:" must
# not skip the Rust jobs. Writes docs_only=true|false to
# $GITHUB_OUTPUT.
set -euo pipefail

out() { echo "docs_only=$1" >> "${GITHUB_OUTPUT:-/dev/null}"; }

# Pull requests and tag pushes always run everything; tags carry
# the release bar.
if [[ "${GITHUB_EVENT_NAME:-}" != "push" ]]; then out false; exit 0; fi
case "${GITHUB_REF:-}" in refs/tags/*) out false; exit 0 ;; esac

before="${PUSH_BEFORE:-}"
after="${PUSH_AFTER:-${GITHUB_SHA:-}}"
# A missing or all-zero base (new branch, rewritten history) means
# the diff cannot be trusted: full CI.
if [[ -z "$before" || "$before" =~ ^0+$ || -z "$after" ]]; then out false; exit 0; fi
if ! git cat-file -e "${before}^{commit}" 2>/dev/null; then out false; exit 0; fi

docs_only=true
while IFS= read -r path; do
  [[ -z "$path" ]] && continue
  case "$path" in
    *.md|sdlc/*|notes/*) ;;
    *) docs_only=false; break ;;
  esac
done < <(git diff --name-only "$before" "$after")

if [[ -z "$(git diff --name-only "$before" "$after")" ]]; then docs_only=false; fi
out "$docs_only"
