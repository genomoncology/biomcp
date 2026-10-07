# 2024 build record — release runners pinned before Ubuntu 26

Landed 2026-10-07, merge f6eca26b3 (branch through 407940cad, main
pre-merged after a base-stale grammar failure). Review verdicts live
on sdlc/tickets/2024-pin-release-runners-before-ubuntu-26.md.

Outcome: the four ubuntu-latest jobs pinned to ubuntu-24.04 (release
order and steps unchanged); build identity through with-build-identity
on host legs and the env-file hand-off on manylinux legs, proven end
to end with a wheel printing its commit; the ubuntu-latest ban reads
whole workflow text; make spec syncs the Python dev environment with
a red-first fresh-worktree proof. Gates: branch CI green on the
main-merged tree (407940cad); yellow lint and spec green at c037bddb3;
the yellow test phase hung twice in the deadline-test module —
forensics captured and filed at
sdlc/issues/2026-10-07-single-backend-deadline-test-can-deadlock.md,
with the identical suite green on the branch CI (the interim
gate-evidence rule recorded there). Record filed at landing.
