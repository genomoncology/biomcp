# 2079 — Add a replay server and golden output tests

Status: OPEN.

Milestone: 0.9.3

Reviews: revision 4e30978, accept

Reviews: revision 4e30978, accept

Reviews: revision 936ba6b, accept

Reviews: revision 936ba6b, accept

## Outcome

Every command's JSON and Markdown output is checked against committed golden files served by one replay server over the recorded provider replies, and a format change is regenerate, read the diff, commit.

## Evidence

- Tests today reach providers through per-page fixture scripts and 128 opaque `jq ... | mustmatch 'true'` assertions; output-format refactors break tests one assertion at a time (`sdlc/designs/2026-10-12-0-9-3-iteration-speed.md`, Caching and fixtures, mustmatch, step 6).
- The cache is on by default; a pre-baked cache was considered and rejected for a replay server over `testdata/sources`.

- Starts from: `sdlc/designs/2026-10-12-0-9-3-iteration-speed.md`, step 6.
- Keeps: mustmatch for reader-facing spec assertions, not extended.
- Changes: claims `tools/replay-server` `tests/golden/test_golden_output.py` `tests/golden/replay_env.py` `tests/golden/conftest.py` `tests/golden/cases/**` `testdata/replay/**` `CHANGELOG.md`; the developer renames these on this ticket to the files the first batch creates, before its first push.
  - One replay server over the recorded replies with a request log and a record mode. It reads `testdata/sources` and never changes it, so the product tickets' recorded replies and `capture-receipts.json` stay theirs; record mode writes new replies under `testdata/replay/`.
  - `tests/golden/replay_env.py` sets every `BIOMCP_*_BASE` to the replay server.
  - `tests/golden/test_golden_output.py` runs each case's command and diffs its JSON and Markdown against the committed files under `tests/golden/cases/`, with `--update` to rewrite them.
  - Seed from the spec pages' commands, gene, variant, trial, article and drug first.
  - One Internal changelog bullet, with the first batch.
- Proof: per batch, every command in the batch has a JSON and a Markdown golden case that fails when its output changes; a renderer change lands with only golden-file updates; the suite runs offline in 60 s or less. Coverage of `src/render` and `src/cli` from the golden suite is reported, not gated.
- Defers: nothing; the remaining commands follow in later batches of this ticket (amendment 2026-10-14).

## Amendment, 2026-10-14

- Per [the decision to delete tests boldly](../decisions/2026-10-14-delete-tests-boldly-and-split-the-suites.md), the first landing is small: the replay server, the `--update` golden test, and golden cases for `get gene`, `get variant` and `search all` (counts-only and full). It lands as one batch with `pm ticket progress 2079 --landed SHA --next TEXT`; the other commands follow in later batches of this ticket.
- 2081 no longer waits for this ticket. Only 2081's last batch, the `search all` unit tests, waits for these golden cases. 2080 still needs the replay server; it puts its golden cases under `tests/golden/spec/`, outside this ticket's claims.
