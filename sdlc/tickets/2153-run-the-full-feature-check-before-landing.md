---
flow: build
priority: 0
deps: []
---

# 2153: Run the full-feature check before landing

Status: WITHDRAWN.

Opened as: 2026-10-17, Ian ruled 2026-10-17 that GitHub runs the release process only and builds run on the build host through pm job, so CI must not grow onto ticket branches..

Milestone: 0.9.3

## Outcome

A ticket cannot land on a tip whose full-feature check has not passed, so main stops turning red on failures that only the full-feature build catches.

## Evidence

- ci.yml runs `full-features` only on pushes to main (`.github/workflows/ci.yml:317-321`). Every routine gate and every ticket-branch CI run builds with `--no-default-features`, so a ticket's green accept tip never includes the full-feature build or its clippy pass.
- 2026-10-17: main went red at 7a5c09d16, 342efb23b and 40483dcab on two `clippy::if_same_then_else` errors in test code that only the full-feature build compiles. On 2026-10-16, main went red four more times after landings that had green branch CI.

## Work

1. Run `full-features` on ticket-branch pushes as well as main, with the same docs-only skip.
2. If the run time on every push is too high, run it on ticket branches only when the branch changes Rust code, and record the measured cost in this ticket.

## Acceptance

1. A ticket-branch push that changes Rust code runs `full-features`, shown by a CI run on this ticket's branch.
2. The changelog carries a bullet for this ticket.
