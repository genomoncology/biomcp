#!/usr/bin/env bash
# Classify a push as docs-only by the files it changed since the
# merge-base with origin/main (2026-09-29 second review): diffing
# against the previous tip let a broken tree look green — a failed
# commit followed by a markdown-only commit skipped every Rust job.
# The base is now the merge-base with origin/main, and only an
# allow-listed set of directories counts as docs, because some
# markdown is executable, embedded or read by Rust tests (spec/*,
# skills/*, src/**, docs/user-guide/cli-reference.md, AGENTS.md —
# spec/surface/mcp.md reads AGENTS.md, so an AGENTS.md-only push
# must run the spec pages; 2026-10-14 amendment).
#
# Ticket 2086 adds a second route for exactly the pm landing shape:
# a ticket-branch push whose only changes since its previous tip are
# under sdlc/, when that previous tip's own ci.yml push run
# concluded success. The green run is the evidence that closes the
# 2026-09-29 hole: a failed tip followed by a record commit still
# runs everything, because its run did not conclude success.
set -euo pipefail

out() { echo "docs_only=$1" >> "${GITHUB_OUTPUT:-/dev/null}"; }

# Pull requests and tag pushes always run everything; tags carry
# the release bar.
if [[ "${GITHUB_EVENT_NAME:-}" != "push" ]]; then
  echo "classify: full CI (event is ${GITHUB_EVENT_NAME:-unset}, not a push)"
  out false; exit 0
fi
case "${GITHUB_REF:-}" in
  refs/tags/*)
    echo "classify: full CI (${GITHUB_REF:-} is a tag; tags carry the release bar)"
    out false; exit 0 ;;
esac

after="${PUSH_AFTER:-${GITHUB_SHA:-}}"
if [[ -z "$after" ]]; then
  echo "classify: full CI (no pushed sha)"
  out false; exit 0
fi

# The base is the merge-base with origin/main: the changes this
# push actually contributes relative to the trunk, not relative to
# whatever the previous tip was (that tip's run may have failed).
base_ref="${PUSH_BASE_REF:-origin/main}"
if ! git rev-parse --verify --quiet "$base_ref" >/dev/null; then
  echo "classify: full CI (base ref $base_ref is missing)"
  out false; exit 0
fi
before="$(git merge-base "$base_ref" "$after" || true)"
# A missing merge-base (unrelated history) means the diff cannot be
# trusted: full CI.
if [[ -z "$before" ]]; then
  echo "classify: full CI (no merge-base with $base_ref)"
  out false; exit 0
fi

# --- Route 1: the allow list over the merge-base diff. ---
docs_only=true
while IFS= read -r path; do
  [[ -z "$path" ]] && continue
  case "$path" in
    # Allow list: repository bookkeeping and plain reference pages
    # nothing compiles, embeds or reads. Everything else — code,
    # workflow, spec/, skills/, any src/ markdown, the compiled CLI
    # reference, AGENTS.md (spec/surface/mcp.md reads it) — runs
    # the full suite.
    # docs/ and README.md are NOT docs-only: docs/charts/ is compiled
    # into the binary (src/cli/chart.rs) and tests/benchmark_cli_
    # structure.rs reads README.md and every docs/**/*.md, so any of
    # them can change Rust-verified behavior (2026-09-30 review).
    # CHANGELOG.md is NOT docs-only either: tests/test_docs_changelog_
    # refresh.py is marked needs_binary, so the repository-contracts
    # lane ("not needs_binary") skips it and only the canonical lane
    # runs it — a changelog-only push must classify as full CI
    # (2026-10-07 review, finding 4).
    sdlc/*|notes/*|.github/*.md) ;;
    *) docs_only=false; break ;;
  esac
done < <(git diff --name-only "$before" "$after")

if [[ -z "$(git diff --name-only "$before" "$after")" ]]; then docs_only=false; fi
if [[ "$docs_only" == true ]]; then
  echo "classify: docs-only by the merge-base route (every path since $base_ref is allow-listed)"
  out true
  exit 0
fi
echo "classify: full CI by the merge-base route (a path outside the allow list changed since $base_ref)"

# --- Route 2: a ticket-record push over a green tip (ticket 2086). ---
# Tried only when route 1 said full CI, the branch is a ticket
# branch, and PUSH_BEFORE is a real ancestor of the pushed tip.
case "${GITHUB_REF:-}" in
  refs/heads/ticket/*|refs/heads/tickets/*) ;;
  *)
    echo "classify: green-tip route not tried (${GITHUB_REF:-unset} is not a ticket branch)"
    out false; exit 0 ;;
esac

push_before="${PUSH_BEFORE:-}"
if [[ -z "$push_before" || "$push_before" =~ ^0+$ ]]; then
  echo "classify: green-tip route not tried (no real previous tip; a new branch reports all zeros)"
  out false; exit 0
fi
if ! git rev-parse --verify --quiet "$push_before" >/dev/null; then
  echo "classify: green-tip route not tried ($push_before is not a commit in this clone)"
  out false; exit 0
fi
if ! git merge-base --is-ancestor "$push_before" "$after"; then
  echo "classify: green-tip route not tried (the previous tip is not an ancestor: force push)"
  out false; exit 0
fi

# Only sdlc/ counts on this route, not the wider allow list: the
# previous tip's run is the proof, and sdlc/ is what pm writes.
records_only=true
while IFS= read -r path; do
  [[ -z "$path" ]] && continue
  case "$path" in
    sdlc/*) ;;
    *)
      records_only=false
      echo "classify: green-tip route refused ($path is outside sdlc/)"
      break ;;
  esac
done < <(git diff --name-only "$push_before" "$after")
if [[ -z "$(git diff --name-only "$push_before" "$after")" ]]; then records_only=false; fi
if [[ "$records_only" != true ]]; then
  out false; exit 0
fi

# The previous tip must have finished a green ci.yml push run. The
# listing is newest-first, so workflow_runs[0] is the newest run; a
# newer in-progress run hides an older green one and forces full
# CI. Any query error, a missing run, or a run that has not
# concluded success means full CI.
run_line="$(gh api "repos/${GITHUB_REPOSITORY:-}/actions/workflows/ci.yml/runs?head_sha=${push_before}&event=push&per_page=1" \
  --jq 'if (.workflow_runs | length) == 0 then "none" else "\(.workflow_runs[0].status)|\(.workflow_runs[0].conclusion)" end' 2>/dev/null || true)"
if [[ -z "$run_line" ]]; then
  echo "classify: green-tip route refused (asking for the previous tip's ci.yml run failed)"
  out false; exit 0
fi
if [[ "$run_line" == "none" ]]; then
  echo "classify: green-tip route refused (the previous tip has no ci.yml push run)"
  out false; exit 0
fi
run_status="${run_line%%|*}"
if [[ "$run_status" != "completed" ]]; then
  echo "classify: green-tip route refused (the previous tip's run is $run_status, not completed)"
  out false; exit 0
fi
run_conclusion="${run_line#*|}"
if [[ "$run_conclusion" != "success" ]]; then
  echo "classify: green-tip route refused (the previous tip's run concluded $run_conclusion)"
  out false; exit 0
fi

echo "classify: docs-only by the green-tip route (every path since $push_before is under sdlc/, and that tip's ci.yml push run concluded success)"
out true
