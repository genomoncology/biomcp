# Clear the pre-0.9.1 backlog hygiene

Filed from the review instruction "then the rest, all before 0.9.1". Umbrella ticket; items may be split out as they are scoped, and this file tracks the list.

## Design

1. Close the remaining should-fix and minor issue files from the first
   review wave that tickets 1234 through 1237 do not already cover:
   - `2026-09-23-mcp-tool-schemas-and-argument-errors.md`
   - `2026-09-23-drug-card-reports-ddinter-not-covered-as-no-interactions.md`
   - `2026-09-23-source-failures-render-as-empty-or-complete-results.md`
   - `2026-09-23-large-futures-and-per-call-runtimes.md`
   - `2026-09-23-test-doc-and-record-hygiene.md`
   - `2026-09-23-pypi-wheels-miss-older-linux-and-linux-arm64.md`
     (scope honestly when read; if this one is a distribution feature
     larger than hygiene, record the decision and bring the trade-off
     with a recommendation instead of expanding silently)
2. Fix the GenCC lease flake per
   `2026-09-23-gencc-lease-test-flakes-under-full-suite-load.md`.
3. Merge the three near-duplicate
   `2026-09-13-raw-ctgov-total-test-*` issue files into one.
4. Older open issue files that describe features or licensing questions
   each get a recorded decision: becomes a ticket, or closes with the
   reason. Record the decisions in this ticket.

## Acceptance

- Every listed file is either closed with a Resolved section or has a
  recorded decision naming its ticket.
- Full yellow gate at the head SHA of each landed change; a record lands.

## Review

- Design review: pending
- Code review: pending
