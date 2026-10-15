# 2055 — Rewrite AGENTS.md for efficient work and remove March

Status: COMPLETE.

Milestone: 0.9.2

Reviews: revision 49049c1, accept

## Outcome

An agent that reads `AGENTS.md` runs targeted tests while it works, runs the full gates once before landing, and treats a slow test as a defect. BioMCP holds no trace of the retired March tool outside its history.

## Evidence

Filed 2026-10-12 from the independent review of the 2052 build, with Ian's approval.

- The 2052 build ran `make test` 12 times, `make spec` 3 times, `make lint` 8 times and targeted nextest 27 times over 4.8 hours. About 3 hours went to the gates, and about 3 minutes went to the live drug check. Two `make test` runs were on unchanged main.
- `AGENTS.md:67-73` says "Run the standard gates directly" and lists `make lint`, `make test` and `make spec` with no word on when or how often. Nothing in the file says to run targeted tests while working, that a full `make test` takes about 15 minutes on a 4-core machine, that a rerun on unchanged code proves nothing, or that a test taking more than a few seconds is a defect.
- `AGENTS.md:46-56` ("Contract quick reference") and `:80-88` ("Skill rail") are bare lists of skill names that are not in this repository.
- March is retired, and the team works through `sdlc/` and pm. Live references remain:
  - `AGENTS.md:104`, `CONTRIBUTING.md:94-95`, `RUN.md:114-117`;
  - `.gitignore:14-15` and `.mcpbignore:3`;
  - `scripts/pre-commit-reject-march-artifacts.sh`, its call at `scripts/pre-commit:62`, and its test `tests/test_pre_commit_reject_march_artifacts.py`;
  - the March allowlist in `tests/test_directory_submission_contract.py:23-151` and the March asserts in `tests/test_upstream_planning_analysis_docs.py:1593-1634` and `tests/test_docs_changelog_refresh.py:369`;
  - the "Runtime Artifacts Stay Ignored" section of `spec/surface/mcp.md:975-982`;
  - `tools/check-quality-ratchet.sh:5`, which writes its output to `.march/reality-check`;
  - the `.march/design-final.md` sentinel in `src/render/json.rs:855`;
  - `architecture/technical/overview.md:350-356`, `request-contract-test-architecture.md:136-137`, `benchmark-cli-ownership-decision.md:26,305` and `live-spec-conversion-target.md:118`, which point at `.march/` files that do not exist in git;
  - `sdlc/planning/onboarding.md:41`, which points at another project's planning file.
- pm reads and lands only `ticket/NNNN-slug` branches (pm ticket 0297 keeps that the only landing form), and pm asked BioMCP on 2026-10-12 to rename its branches to that form. `.github/workflows/ci.yml:9` runs CI only on `main` and `tickets/**`, and `AGENTS.md:109` says to push to `tickets/*`. A branch renamed to pm's form gets no CI run, so it can never meet the merge rule or pm's landing check.
- Ignored `.march/` folders still sit in the BioMCP checkouts on the coordinator's machine and the build host.

- Starts from: main `1268b7e11`.
- Keeps: the release, CI and merging, and source licensing sections of `AGENTS.md`; the credential and forbidden-artifact scans in `scripts/pre-commit`; `src/transform/article/federation.rs:449`, where "march" is the month.
- Changes: shorten `AGENTS.md` and give it one working section in BioMCP's own commands, matching the team's developer rules: while working, run only the tests for what changed (`cargo nextest run --no-default-features -E 'test(...)'`); before landing, run `make lint`, `make test` and `make spec` once on the final branch, and rerun only when the code changed, since CI runs them again; a test that takes more than a few seconds is a defect to fix; check live behavior with one pass of the built binary over the names under test; run one Rust build at a time on a machine. Delete the two bare lists. Delete every live March reference above, including the hook script, its test, its allowlist asserts and the spec section; point the quality ratchet at an ignored folder under `target/`; use a neutral local path as the sentinel; delete the `.march/` folders from the checkouts. Add `ticket/**` to the CI branch filter and say `ticket/NNNN-slug` in `AGENTS.md`; keep `tickets/**` until no open branch uses it.
- Proof: `git grep -iE '\.march|march-runtime|march (run|kickoff|worktree|lane)'` outside `CHANGELOG.md`, `sdlc/records/`, `sdlc/tickets/archive/` and `architecture/experiments/` finds nothing; `make lint`, `make test` and `make spec` pass once on the final branch; `AGENTS.md` names no machine or private tool.
- Defers: March mentions in released changelog entries, archived tickets, records and old experiment notes, which describe past work.

## Evidence (2026-10-12)

Branch `tickets/2055-work`, code head `165afa553` (this evidence commit follows it; it changes only this file).

`AGENTS.md`: keeps the release, CI-and-merging, and source-licensing sections; deletes the two bare skill-name lists; adds one Working section (targeted `cargo nextest run --no-default-features -E 'test(...)'` while working; `make lint`, `make test` and `make spec` once on the final branch, rerun only when the code changed; a slow test is a defect; one live pass with the built binary; one Rust build at a time on a machine); says `ticket/NNNN-slug`; names no machine and no private tool (`grep -ioE "yellow|beelink|\bm5\b|\bpm\b|march|\byr\b|jarvis|\bpi\b" AGENTS.md` → no output).

Removed every listed live March reference: the `AGENTS.md` hygiene line, the `CONTRIBUTING.md` and `RUN.md` hook paragraphs, `.gitignore` (`.march/*`, `.march-runtime/`), `.mcpbignore`, `scripts/pre-commit-reject-march-artifacts.sh` with its call in `scripts/pre-commit` (the credential and forbidden-artifact scans stay), `tests/test_pre_commit_reject_march_artifacts.py`, the March allowlist in `tests/test_directory_submission_contract.py`, the March asserts in `tests/test_upstream_planning_analysis_docs.py` and `tests/test_docs_changelog_refresh.py`, and the `spec/surface/mcp.md` "Runtime Artifacts Stay Ignored" section; that page's root-guide block now pins the Working strings (`mustmatch lint` clean), and its make-check section no longer credits March. The quality ratchet writes to `target/quality-ratchet` (ignored); the `src/render/json.rs` sentinel is `.local/design-final.md`; the four architecture `.march/` pointers and `sdlc/planning/onboarding.md`'s pointer into another project's planning file are gone. `src/transform/article/federation.rs:449` (the month) is untouched. Also: `ticket/**` joins `tickets/**` in the `ci.yml` push filter (actionlint clean via `tools/bootstrap-lint-tools`); `tests/test_source_package_boundary.py` counts 1,418 package files for the two deleted files; and pm's 0.9.2 build order (2052, 2053, 2055, then 2054 after the tag) is recorded in `sdlc/plan.md`.

Ignored `.march/` state deleted from the coordinator checkout (`~/workspace/repos/biomcp/.march` held only ratchet output); the build-host checkout has none (`find ~/workspace/repos/biomcp -maxdepth 2 -name ".march*"` → empty). No commit was made outside this worktree.

Proof grep on head `165afa553`:

```
git grep -n -iE '\.march|march-runtime|march (run|kickoff|worktree|lane)' -- . ':!CHANGELOG.md' ':!sdlc/records/' ':!sdlc/tickets/archive/' ':!architecture/experiments/'
```

Output: six lines, every one inside this ticket file's own text (the spec quoting the references it ordered removed). The same command with `':!sdlc/tickets/2055-*'` added prints nothing; once this ticket archives under `sdlc/tickets/archive/`, the command as written prints nothing.

Targeted tests while working (build host): `cargo nextest run --no-default-features -E 'test(structured_error_messages_omit_external_and_local_details)'` → 1 passed, 0.010s; the four edited Python modules → 66 passed in 1.20s; the package-boundary count and worktree-path tests under the lane `TMPDIR` → 2 passed.

Gates, once each on the final branch state (build host): `make lint` → pass; `make test` → pass; `make spec` → pass. Two earlier `make test` runs do not count: the first failed the two package-boundary count asserts (1,420 → 1,418, fixed above), and the rerun after that fix hit one load-flaky failure (`rmcp_client_contract raw_and_typed_article_query_validation_converges_before_provider_work`, 60.17s under the full suite; it passes alone in 11.73s and the next full run was green).

Live pass with the built binary: `target/spec/biomcp --version` → `biomcp 0.9.2+g165afa55`; `target/spec/biomcp search article --gene BRAF --limit 1` answered one live public-upstream query with rendered output.

Defects noted, not fixed here (outside this ticket's scope): `raw_and_typed_article_query_validation_converges_before_provider_work` takes 11.7s alone and 60s under the full suite, far over the few-second bar this ticket installs, and it failed once under load; `tests/test_source_package_boundary.py` runs about five minutes, dominated by the offline package compile.
