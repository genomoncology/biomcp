# 2055 — Rewrite AGENTS.md for efficient work and remove March

Status: OPEN.

Milestone: 0.9.2

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
- Ignored `.march/` folders still sit in the BioMCP checkouts on the coordinator's machine and the build host.

- Starts from: main `1268b7e11`.
- Keeps: the release, CI and merging, and source licensing sections of `AGENTS.md`; the credential and forbidden-artifact scans in `scripts/pre-commit`; `src/transform/article/federation.rs:449`, where "march" is the month.
- Changes: shorten `AGENTS.md` and give it one working section in BioMCP's own commands, matching the team's developer rules: while working, run only the tests for what changed (`cargo nextest run --no-default-features -E 'test(...)'`); before landing, run `make lint`, `make test` and `make spec` once on the final branch, and rerun only when the code changed, since CI runs them again; a test that takes more than a few seconds is a defect to fix; check live behavior with one pass of the built binary over the names under test; run one Rust build at a time on a machine. Delete the two bare lists. Delete every live March reference above, including the hook script, its test, its allowlist asserts and the spec section; point the quality ratchet at an ignored folder under `target/`; use a neutral local path as the sentinel; delete the `.march/` folders from the checkouts.
- Proof: `git grep -iE '\.march|march-runtime|march (run|kickoff|worktree|lane)'` outside `CHANGELOG.md`, `sdlc/records/`, `sdlc/tickets/archive/` and `architecture/experiments/` finds nothing; `make lint`, `make test` and `make spec` pass once on the final branch; `AGENTS.md` names no machine or private tool.
- Defers: March mentions in released changelog entries, archived tickets, records and old experiment notes, which describe past work.
