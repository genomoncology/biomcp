# 2086 — Skip the full gates for a push that adds only ticket records to a green tip

Status: COMPLETE.

Milestone: 0.9.3

Reviews: revision b45d3c4, accept

## Outcome

A ticket-branch push whose only change since a tip with a green CI run is under `sdlc/` finishes CI in a few minutes, and every other push runs the full gates as today.

## Evidence

- `pm ticket review NNNN accept` pushes one commit that adds two lines to the ticket file, and landing needs a green run on that exact tip. `scripts/ci-classify-push.sh` diffs the push against its merge-base with `origin/main`, so on a branch with code changes that commit runs every gate again. On 2026-10-14 the 2069 branch ran full CI five times; three of those runs were on "Record 2069 review: accept" commits that each changed only `sdlc/tickets/2069-record-the-0-9-3-baseline.md`, at 24 to 30 minutes each. Every landing pays at least one such run.
- The merge-base rule exists for a reason (the 2026-09-29 second review, quoted in the script's header): diffing against the previous tip let a failed commit followed by a markdown-only commit skip every Rust job. Requiring the previous tip's own run to be green closes that hole.
- `repository-contracts` is not gated on `docs_only`, so the ticket-format and changelog-coverage checks that read `sdlc/` still run on a skipped push.

- Starts from: `scripts/ci-classify-push.sh` (the `docs_only` allow list and the merge-base diff); the `changes` job in `.github/workflows/ci.yml` (lines 29-45), whose env passes `PUSH_BASE_REF` and `PUSH_AFTER`; the workflow's top-level `permissions: contents: read` (line 13); `tests/test_ci_classify_push_script.py`, which runs the real script against scratch repositories; `tests/test_ci_workflow_contract.py`.
- Keeps: the merge-base route and its allow list; full CI for pull requests, tags, pushes to `main`, and any push that touches a path outside the allow list; the `repository-contracts` job on every push.
- Changes: claims `scripts/ci-classify-push.sh` `.github/workflows/ci.yml` `tests/test_ci_classify_push_script.py` `tests/test_ci_workflow_contract.py` `CHANGELOG.md`.
  - Add a second route to the script, tried only when the merge-base route says full CI, the ref is `refs/heads/ticket/*` or `refs/heads/tickets/*`, and `PUSH_BEFORE` is a real commit (not all zeros, and an ancestor of `PUSH_AFTER`). It sets `docs_only=true` when every path in `git diff --name-only PUSH_BEFORE PUSH_AFTER` is under `sdlc/` and the newest `ci.yml` push run for `PUSH_BEFORE` concluded `success`. The script asks for that run through `gh api "repos/$GITHUB_REPOSITORY/actions/workflows/ci.yml/runs?head_sha=$PUSH_BEFORE&event=push"`. Any error, a missing run, or a run still in progress means full CI.
  - Only `sdlc/` counts on this route, not the wider allow list, because the previous tip's run is the proof and `sdlc/` is what pm writes.
  - In the `changes` job: pass `PUSH_BEFORE: ${{ github.event.before }}` and `GH_TOKEN: ${{ github.token }}`, and give the job `actions: read` in its own `permissions` block.
  - The script prints which route decided and why, so a reader of the log sees the reason for a skip.
  - One Internal changelog bullet.
- Proof: new cases in `tests/test_ci_classify_push_script.py` with a fake `gh` on `PATH`: an sdlc-only push over a green tip gives `docs_only=true`; the same push over a failed, missing or in-progress tip gives false; a push touching any file outside `sdlc/` gives false; a force push whose before is not an ancestor gives false; a push to `main` gives false. All fail on current main except the false cases. Then on a scratch `ticket/` branch: push a code change, wait for green, push an sdlc-only commit, and record both runs' durations in the review.
- Defers: making pm record reviews without a branch commit (a pm question, asked by mail); the CI restructure (2075).
- Order: right after 2083. It shares no file with 2070, 2083 or the product tickets; it shares `.github/workflows/ci.yml` with 2074 to 2076, which come later.

## Amendment 2026-10-14: AGENTS.md leaves the docs-only allow list

`spec/surface/mcp.md` ("Root Agent Guide Declares The Contract") reads `AGENTS.md`, so an `AGENTS.md`-only push must run the spec pages. On 2026-10-14 commit 393737eae changed only `AGENTS.md`, classified as docs-only, skipped the spec pages on its branch, and turned main red until ec4029345.

- Changes: remove `AGENTS.md` from the merge-base route's allow list in `scripts/ci-classify-push.sh`, and update the allow-list string `tests/test_ci_workflow_contract.py` strips.
- Proof: a case in `tests/test_ci_classify_push_script.py` where a push changing only `AGENTS.md` gives `docs_only=false`; it fails on current main.

- Built on `ticket/2086-ci-record-skip` from main at `a81c61028` (green again after `ec4029345`). `scripts/ci-classify-push.sh` runs the merge-base route first and, only when that says full CI, the green-tip route: ticket branch (`refs/heads/ticket/*` or `refs/heads/tickets/*`), `PUSH_BEFORE` a real ancestor (all zeros, a missing sha, or a non-ancestor force push each refuse), every path in `git diff --name-only PUSH_BEFORE PUSH_AFTER` under `sdlc/`, and the newest `ci.yml` push run for the previous tip (`gh api "repos/$GITHUB_REPOSITORY/actions/workflows/ci.yml/runs?head_sha=$PUSH_BEFORE&event=push&per_page=1"`) `completed` with conclusion `success`; any error, missing run, or unfinished run runs full CI. Every exit prints the deciding route and reason.
- Proof (red on main, `a81c61028`): `tests/test_ci_classify_push_script.py` against main's script — 5 failed, 25 passed. Failing: `test_sdlc_only_push_over_green_tip_classifies_true` (both `ticket/*` and `tickets/*` refs), `test_second_route_counts_only_sdlc_paths`, `test_force_push_before_not_ancestor_runs_full_ci` (main prints no route), `test_agents_md_only_push_runs_full_ci` (main still allows `AGENTS.md`). All the false cases (failed/cancelled/missing/in-progress/queued tip, gh error, non-ticket branch, all-zeros before, no `PUSH_BEFORE`) pass on main.
- Proof (tests, Linux check host, `40fe4c85a`): `tests/test_ci_classify_push_script.py` (19 cases incl. the parametrized green-tip set, both branch spellings, the fake `gh` recording the URL it was asked) and `tests/test_ci_workflow_contract.py` (now pins `PUSH_BEFORE: ${{ github.event.before }}`, `GH_TOKEN: ${{ github.token }}`, and the changes job's `permissions: contents: read, actions: read`) — 28 passed. `make lint` green on the Linux check host at `40fe4c85a` (one earlier ratchet run failed and a rerun of the same command on the same tree passed; the recorded state is green). `make test`'s changelog lane covers the new bullet through CI.
- Beyond the claimed files: `tests/test_docs_changelog_refresh.py` (the 0.9.3 dev-cycle ticket set gains 2086; CI run 38636480337 on `1843c9fd` failed on exactly that and commit `40fe4c85` closed it).
- Proof (CI): run 38640267374 on `40fe4c85a23f267a40a724a50a9ee338a2e3e7b8` green, every job, 27m15s wall (21:26:24Z–21:53:39Z). The changes job log shows the route print working in production: `classify: full CI by the merge-base route (a path outside the allow list changed since origin/main)` then `classify: green-tip route refused (tests/test_docs_changelog_refresh.py is outside sdlc/)` — the code push never skips.
- Proof (durations, this branch's own pushes): the code push `40fe4c85` ran every gate in 27m15s; the record push that appends this evidence runs against that green tip through the new route — its duration follows in the next evidence commit.
- Proof (durations, closed): the record push `66ec6533` (run 38643865172, this evidence) classified `docs_only=true` by the green-tip route — the changes log prints `classify: docs-only by the green-tip route (every path since 40fe4c85a23f267a40a724a50a9ee338a2e3e7b8 is under sdlc/, and that tip's ci.yml push run concluded success)` — and finished green in 2m31s (21:54:56Z–21:57:27Z) with `canonical-gates`, `stress-lane`, `release-panic`, `full-features`, `generated-sources`, and `windows-contracts` all skipped and only `repository-contracts` running, against the code push's 27m15s on the same branch.
