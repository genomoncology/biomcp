---
base: b16c57d3
head: 2159551c
---

Made the changelog gate see every ticket and the workflow tests guard
every step, from the release-gate follow-ups issue.

Ticket discovery is now the union of two git scans between the
previous stable tag and the release tag: merge subjects matching
`tickets/NNNN-` and `sdlc/records/NNNN-*.md` files added in the
range. Neither alone is complete — a record can lag a merge, and a
merge subject can be rewritten or absent — so the gate fails closed
on the union; the dry run's eight silently-skipped tickets
(1219-1224, 1236, 1237, 1247) all surface. The union is proven
against real git history (a temp repository with two tags), not a
canned response. Bullet acceptance now strips every bare number
token and separator after the ticket marker and requires at least
three word characters, so a bare list of ticket numbers no longer
counts as a described bullet while every real Unreleased bullet
passes. The fake-git harness dispatches by subcommand, so adding the
diff call cannot shift responses.

The workflow provenance tests became a data-driven contract over the
parsed YAML for pypi-build, wheel-smoke, and docs-live plus the
trigger block: no run step swallows its own failure (`continue-on-error`
banned, the two protoc installer steps carved out; job-level
`continue-on-error` banned too), no banned escape string (`|| true`,
`|| :`, `; exit 0`, `if: ${{ false }}`), no step switched off by a
false `if`, matrix entries (runner, container, target, artifact)
pinned exactly, the trigger block push-only, the floor check present
in both wheel legs, and the three content pins the issue demanded by
name: the panic `return 1`, the not-found exit check, and the runtime
floor smoke's `if:` condition. Twelve parametrized pipeline mutations
each flip a specific assertion; the version-check escape set gained
the three previously uncaught strings.

Evidence: design REJECT once (records-only discovery would have
skipped ticket 1246 itself; the blanket continue-on-error ban broke
the protoc steps) — folded to the union and the run-step scoping,
re-reviewed ACCEPT; code review ACCEPT with two P2s both folded (the
real-git fixture test; the job-level escape assert); yellow gate at
2159551c — lint, test, and spec OK.

Residuals: the fifteen missing changelog bullets are release-prep
work, listed in ticket 1253; the build job's steps stay outside the
three-job contract scope (recorded, not demanded by the issue).

## Follow-ups (2026-09-26)

The review's eight escapes closed, each with its specific failure
message asserted: continue-on-error in any spelling at step and job
level; || exit 0, set +e, and trap 'exit 0' in any spacing or quote
form; every switch-off if: shape via a closed allowlist of the two
conditions the guarded jobs actually use (a new condition needs an
explicit, reviewable edit); shell: bash {0} banned; the docs-live
timeout branch pinned to exit 1; and every job's if: asserted by
exact equality with !cancelled() banned outside container-publish —
so pypi-publish can no longer publish after a failed smoke. The
changelog gate strips boilerplate label words (tickets, see, fixed,
changed, updated, added) before counting word characters, so
"- Tickets 1226, 1227" and "- see 1226" fail while every real bullet
passes. Two adversarial forms are pinned as committed mutations
(no-space ${{true}}, double-quoted trap); the honest residuals
(bash-semantics wrappers like `if false; then`, a dropped shell:
declaration, and the legitimate final exit 0) are recorded. Code
review ACCEPT with two P2s folded (the allowlist trimmed to in-scope
conditions; the stoplist widened); yellow gate at 60e75532 — lint,
test, and spec OK.
