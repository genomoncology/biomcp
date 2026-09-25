# Release gate follow-ups after ticket 1234

Filed 2026-09-24 from the review of ticket 1234 (merge 91da17e6). The reviewer ran a v0.9.1 dry run in a scratch copy: every version set to 0.9.1, Unreleased renamed to `## 0.9.1 — 2026-09-25`, tagged. The version gate and `check-version-sync.sh` passed. Most of the 1233 findings are fixed for real, and the four named mutations are now caught.

## The changelog lacks bullets for 15 merged tickets

The coverage gate exited 1 in the dry run. Tickets 1226-1235, 1239-1241, 1244, and 1246 have no bullet. The gate is right. Write the bullets during release prep.

## Ticket discovery misses tickets

`scripts/check-changelog-coverage.py:12` sees only merge subjects containing `tickets/NNNN-`. Tickets 1219, 1220, 1222, 1223, 1224, 1236, 1237, and 1247 landed as "Merge ticket 1236 ...", or with no merge commit, and the gate skips them silently. Take tickets from the `sdlc/records/NNNN-*` files added between the two tags. That list is complete for this range.

## A bare number list still passes

`:92` accepts `- 1226, 1227, 1228` as described text. Remove every ticket-like number before testing the leftover text and require a minimum word count.

## The workflow tests guard only the version-check job

On a scratch copy, 17 of 18 edits outside that job left the tests green:

- `continue-on-error` on wheel-smoke or docs-live
- `|| true` on the floor check, or removing it
- running the ARM smoke on an x86 runner, or dropping the ARM wheel build
- `if: false` on the runtime floor smoke
- deleting the not-found exit check, or the `return 1` after the panic message
- `|| :`, `; exit 0`, or `if: ${{ false }}` on the version-check steps
- restoring a `release: published` trigger

Apply the no-escape assertions to every step of pypi-build, wheel-smoke, and docs-live. Pin each matrix entry's runner, container, and artifact. Pin the trigger block. The reviewer's mutation scripts can seed the test list.

## Recorded, not a defect

Pre-release tags are rejected by decision rather than mapped to PEP 440. That is fine for 0.9.1.

## Resolved

Ticket 1250. Discovery is the union of records-added and
merge-subject scans (fail closed); a bare number list fails; the
pipeline contract covers every step of pypi-build, wheel-smoke, and
docs-live plus the trigger, with twelve mutations each flipping an
assertion; the three content pins landed. The fifteen bullets are
listed in ticket 1253 for release prep. See
`sdlc/records/1250-make-the-changelog-gate-and-workflow-tests-total.md`.
