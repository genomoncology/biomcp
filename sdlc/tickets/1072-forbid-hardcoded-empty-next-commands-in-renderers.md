---
flow: build
priority: 5
deps: ["1069"]
---

# Forbid hardcoded empty next-commands in renderers

The markdown author card's dropped follow-ups (1069) came from one line:
`next_commands: vec![]` hardcoded in `src/render/markdown/author.rs:178`
— the only such site in the codebase today, but nothing stops the next
one. This ticket adds the cheap mechanical guard while 1071 builds the
full cross-surface contract: the quality ratchet greps `src/render` for
the literal `next_commands: vec![]` and fails the lint lane if it
reappears.

## Done when

- `tools/check-quality-ratchet.sh` (or the lint lane it feeds) rejects
  any `next_commands: vec![]` literal under `src/render`, with a message
  naming the one-source policy and the alternative (take the commands
  from the shared source the JSON `_meta` uses).
- The guard fails on today's pre-1069 code (the author.rs site) and
  passes once 1069 lands — the dep ordering enforces this.
- The ratchet's README or comment names the bug that earned it (1069)
  so a future reader knows why the line exists.
