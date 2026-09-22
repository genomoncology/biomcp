---
base: a172d49a
head: 73bfc267
---

Added a release gate that fails while biomcp.org serves documentation older
than the tag.

The site keeps exactly one revision witness file and deletes the previous one
on every deploy, so the tag commit's witness can vanish as soon as main
advances, and a naive equality check would block correct releases. The docs
build now also writes `__biomcp_revision__/latest.txt` with the same sha, which
survives the retention rule. A new `docs-live` job resolves the tag commit with
`gh api .../commits/$TAG --jq .sha`, fetches the pointer with cache-busting
headers and bounded retries inside a 600-second window, and runs
`scripts/check-docs-live-revision.py`, which passes on `identical` and `ahead`
and fails on `behind`, `diverged`, and any `gh` error. The job is in the
`needs` of `pypi-publish`, `homebrew-tap`, and `container-publish`, and the
container job's `always()` condition also requires its success. It carries no
`container_only` gate, so it runs on both triggers and a container-only
backfill also waits for a current site. The helper is checked out from the
default branch so an older-tag re-run still finds it, and the fetch has
connection and total timeouts.

Evidence: actionlint clean; 43 focused Python tests pass, including the new
fake-`gh` gate test over equal, descendant, behind, divergent, and gh-error
cases and the pointer rotation contract; mutation checks show that dropping any
publisher's gate, the checkout ref pin, or the fetch timeouts fails a test.

Reviews: the design review rejected the first draft and required the stable
pointer plus gating all three publishers; the code review accepted with two
fixes, both applied.

Residual: the fail branch needs a stale live site or a hosted dispatch to be
proven end to end, and the pointer only appears on biomcp.org after this lands
and the docs workflow deploys. The architecture overview's release paragraph
still omits the gate; the runbook is the source of record.
