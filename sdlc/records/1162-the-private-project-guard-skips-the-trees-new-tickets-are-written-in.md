---
flow: build
priority: 8
status: complete
---

# The private-project guard scans the trees where new tickets are written

## Outcome

The existing documentation-consistency audit now scans open numbered Markdown
tickets directly under `sdlc/tickets/` and every Markdown draft under
`sdlc/tickets/drafts/`. A top-level ticket is historical when a direct
completion-record filename matches `NNNN-*.md` with the same four-digit ID,
regardless of slug.
Drafts remain current regardless of record membership. Completion records,
record-backed top-level tickets, malformed top-level names, and archived
tickets remain outside the scan.

The diagnostic is collected in stable relative-path, line, and configured
marker order. One temporary-tree regression exercises existing current roots,
an open direct ticket, direct and nested drafts, a same-ID/different-slug draft
and record, a record-backed top-level ticket, malformed top-level ticket and
completion-record filenames, an archived ticket, and records containing
tracked markers.

## Evidence

- Red: the new focused regression failed with `NameError` before the
  lifecycle-aware scanner existed.
- Focused green: the new regression passed, then the complete documentation
  consistency module passed 19 tests both before and after ticket promotion.
- `make lint` passed, including Python, Bash/workflow, Rust, license, advisory,
  and quality-ratchet checks.
- `make test` passed 3,152 Rust tests with 30 skipped, 902 Python tests with 3
  skipped, and the strict documentation build.
- `make spec` passed all routine and static specification suites.
- `git diff --check` passed. No canonical lifecycle script changed.
- Primary integration repeated the focused 19-test module and the canonical
  `make lint`, `make test`, and `make spec` gates with the same passing counts.

## Review

- Design review: accepted before implementation.
- Initial code review: rejected the over-broad ticket and record filename
  parsing; remediation restored the accepted exact `NNNN-*.md` boundaries and
  added malformed-filename regressions.
- Independent code re-review: accepted the remediation with no findings after
  confirming exact full-match parsing, malformed-name coverage, preserved
  draft precedence, unchanged canonical scripts, and a clean worktree.

## Boundary

The three literal markers are unchanged. The audit does not scan completion
records or archived tickets, rewrite historical material, broaden marker
matching, or change canonical lifecycle scripts.
