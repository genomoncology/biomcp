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
- Changes: one replay server over `testdata/sources` with a request log and a record mode; a helper that sets every `BIOMCP_*_BASE` to it; a golden test that runs each command and diffs JSON and Markdown against committed files, with `--update`; seed from the spec pages' commands, gene, variant, trial, article and drug first.
- Proof: coverage of `src/render` and `src/cli` from the golden suite alone against today's 70% and 48% outside-in; a renderer change landed with only golden-file updates; the suite runs offline.
- Defers: the remaining entities, which follow in 2081.
