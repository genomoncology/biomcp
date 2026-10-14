# 2079 — Add a replay server and golden output tests

Status: OPEN.

Milestone: 0.9.3

## Outcome

Every command's JSON and Markdown output is checked against committed golden files served by one replay server over the recorded provider replies, and a format change is regenerate, read the diff, commit.

## Evidence

- Tests today reach providers through per-page fixture scripts and 128 opaque `jq ... | mustmatch 'true'` assertions; output-format refactors break tests one assertion at a time (`sdlc/designs/2026-10-12-0-9-3-iteration-speed.md`, Caching and fixtures, mustmatch, step 6).
- The cache is on by default; a pre-baked cache was considered and rejected for a replay server over `testdata/sources`.

- Starts from: `sdlc/designs/2026-10-12-0-9-3-iteration-speed.md`, step 6.
- Keeps: mustmatch for reader-facing spec assertions, not extended.
- Changes: claims `tests/golden/**` `tools/replay-server` `CHANGELOG.md`, renamed on this ticket to the files the first batch creates before its first push. One replay server over the recorded replies in `testdata/sources` with a request log and a record mode; a helper that sets every `BIOMCP_*_BASE` to it; a golden test that runs each command and diffs JSON and Markdown against committed files, with `--update`; seed from the spec pages' commands, gene, variant, trial, article and drug first.
- Proof: coverage of `src/render` and `src/cli` from the golden suite alone against today's 70% and 48% outside-in; a renderer change landed with only golden-file updates; the suite runs offline.
- Defers: nothing; the remaining commands follow in later batches of this ticket (amendment 2026-10-14).

## Amendment, 2026-10-14

- Per [the decision to delete tests boldly](../decisions/2026-10-14-delete-tests-boldly-and-split-the-suites.md), the first landing is small: the replay server, the `--update` golden test, and golden cases for `get gene`, `get variant` and `search all` (counts-only and full). It lands as one batch with `pm ticket progress 2079 --landed SHA --next TEXT`; the other commands follow in later batches of this ticket.
- 2081 no longer waits for this ticket. Only 2081's last batch, the `search all` unit tests, waits for these golden cases. 2080 still needs the replay server.
