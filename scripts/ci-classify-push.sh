#!/usr/bin/env bash
# Classify a push as docs-only by the files it changed since the
# merge-base with origin/main (2026-09-29 second review): diffing
# against the previous tip let a broken tree look green — a failed
# commit followed by a markdown-only commit skipped every Rust job.
# The base is now the merge-base with origin/main, and only an
# allow-listed set of directories counts as docs, because some
# markdown is executable, embedded or read by Rust tests (spec/*,
# skills/*, src/**, docs/user-guide/cli-reference.md).
set -euo pipefail

out() { echo "docs_only=$1" >> "${GITHUB_OUTPUT:-/dev/null}"; }

# Pull requests and tag pushes always run everything; tags carry
# the release bar.
if [[ "${GITHUB_EVENT_NAME:-}" != "push" ]]; then out false; exit 0; fi
case "${GITHUB_REF:-}" in refs/tags/*) out false; exit 0 ;; esac

after="${PUSH_AFTER:-${GITHUB_SHA:-}}"
if [[ -z "$after" ]]; then out false; exit 0; fi

# The base is the merge-base with origin/main: the changes this
# push actually contributes relative to the trunk, not relative to
# whatever the previous tip was (that tip's run may have failed).
base_ref="${PUSH_BASE_REF:-origin/main}"
if ! git rev-parse --verify --quiet "$base_ref" >/dev/null; then
  out false; exit 0
fi
before="$(git merge-base "$base_ref" "$after" || true)"
# A missing merge-base (unrelated history) means the diff cannot be
# trusted: full CI.
if [[ -z "$before" ]]; then out false; exit 0; fi

docs_only=true
while IFS= read -r path; do
  [[ -z "$path" ]] && continue
  case "$path" in
    # Allow list: repository bookkeeping and plain reference pages
    # nothing compiles, embeds or reads. Everything else — code,
    # workflow, spec/, skills/, any src/ markdown, the compiled CLI
    # reference — runs the full suite.
    # docs/ and README.md are NOT docs-only: docs/charts/ is compiled
    # into the binary (src/cli/chart.rs) and tests/benchmark_cli_
    # structure.rs reads README.md and every docs/**/*.md, so any of
    # them can change Rust-verified behavior (2026-09-30 review).
    sdlc/*|notes/*|CHANGELOG.md|AGENTS.md|.github/*.md) ;;
    *) docs_only=false; break ;;
  esac
done < <(git diff --name-only "$before" "$after")

if [[ -z "$(git diff --name-only "$before" "$after")" ]]; then docs_only=false; fi
out "$docs_only"
