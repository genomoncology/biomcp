---
flow: quickfix
priority: 10
---
# Fix pre-existing make spec-pr and make test-contracts gate failures blocking 287

biomcp/287 completed its own work perfectly (25/25 checkpoint, 1 commit, clean worktree, witness passed) but aborted at the `make check` gate because the repo-wide validation surface is red on issues **unrelated to 287's scope**:

Completed under March on 2026-04-24, as March ticket 294. Imported as history when BioMCP
moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/294-fix-pre-existing-make-spec-pr-and-make-test-contracts-gate-failures-blocking-287

The landed commit range could not be recovered from git, so no
record accompanies this entry. That is a known gap for the
earliest tickets, not a sign the work is missing.
