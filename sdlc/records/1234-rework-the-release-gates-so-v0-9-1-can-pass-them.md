---
base: 96dfa826
head: b9a3be15
---

Reworked the release gates so v0.9.1 can pass them, from the issue filed
2026-09-23 after the independent review of ticket 1233's gates.

The Release workflow now triggers on `push: tags: ['v*']` with
`workflow_dispatch` retained for published-release container backfills,
and `TAG` resolves as `inputs.tag || github.ref_name`. A `version-check`
job checks out the tag ref with full history, runs the new
`scripts/check-release-versions.py` (stable `^v` tags only, tag equal to
the committed Cargo and pyproject versions, `check-version-sync.sh`
invoked for every metadata file, pre-releases rejected with a clear
message) and the reworked `scripts/check-changelog-coverage.py` (reads
the release heading's section first and falls back to Unreleased to the
next heading or end of file; tickets discovered only from
`Merge ... tickets/NNNN-` subjects via `git log`, never the
250-commit-capped compare API; each ticket needs a described bullet or an
internal-only marker). A `create-draft` job creates the GitHub release as
a draft so `gh release upload` has a target, and a final `publish-release`
job publishes the release and moves the container `latest` pointer only
after every publisher succeeds; the guarantee is that the GitHub release,
its assets, and `latest` stay private until then, while PyPI and the tap
keep today's earlier-channel exposure, now recorded rather than implied.
The container job's condition is exactly the reviewed branch-local form:
push requires every need green, dispatch requires the dispatch gates
green, the push-only needs skipped, and `container_only` set. Permissions
are empty at the top and granted per job; concurrency is tag-keyed;
third-party actions are pinned to admitted immutable revisions; the
wheel smoke runs on all four built wheels with exit-0 deep paths, the
not-found adverse-event fallback with its exact text, JSON-mode legs, and
failure on exit 101 and any crash.

Tests pin the whole surface: one mutation test per needs edge in the
reviewed adjacency, gate-neutering mutations (`continue-on-error`, `if:`
escapes, `|| true`), the three container-condition clause mutations, and
behavior tests for both scripts including the fake-git changelog cases
and the version script's stable-only rejection. The dispatch input
description still says "upload assets"; the runbook is authoritative and
names the contract.

Process: this ticket went through seven design reviews (six REJECTs; the
sixth caught a lost edit) and one code review. The harness outage that
blocked reviews earlier was repaired in place (pi-subagents pinned at
0.70.1, `@earendil-works/pi-agent-core` installed, two bootstrap
symlinks) and every verdict from the fourth design review onward ran on
the repaired stack.

Evidence: code review ACCEPT with the `packages: write` note (folded
into the ticket's item 12 before merge); yellow at b9a3be15 — `make
lint` OK, `make test` OK (1,018 passed, 3 skipped; 0 FAILED), `make
spec` OK. The first gate run at 330a5740 failed four contract pins — the
runbook rewrite had dropped the version-metadata sentence the quality
ratchet reads from the package files, two pytest pins still expected the
`release: published` trigger, and the docker-image spec page pinned the
old per-job concurrency group — all fixed in b9a3be15 with the ratchet
and 72 focused tests green locally first.

Residuals: PyPI and the Homebrew tap can publish before the GitHub
release goes public if a later job fails (unchanged exposure, recorded).
The four-platform smoke shares live providers, so a provider outage
fails the release, accepted as fail-closed. Everything here gets its
first live exercise at the 0.9.1 tag.
