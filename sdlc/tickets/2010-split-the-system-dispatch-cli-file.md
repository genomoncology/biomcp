# 2010 — Split the system dispatch CLI file

Status: complete. Absorbed by ticket 1304
Landed: f6eca26b3
Milestone: 0.9.2

## Outcome

`src/cli/system/dispatch.rs` returns under the 700-line CLI cap and its
allowlist entry retires.

## Evidence

- Starts from: ticket 1304's line-cap split of the system CLI.
- Keeps: the dispatch behavior.
- Changes: the split retired this ticket's allowlist entry.
- Proof: the allowlist entry is gone at the 1304 landing; that fold split the who-sync reporting into src/cli/system/who_sync.rs and moved its tests to tests/who_sync.rs, both parents under the cap and the allowlist empty again.
- Defers: nothing.

Ticket 1304's honest sync reporting grew the file to 726 lines; the CLI
line-cap ratchet failed CI. The allowlist carries the overage with this
ticket as the follow-up.

## Change detail

1. Extract the who-sync reporting block (or the natural seam the file
   offers) into a submodule, mirroring the query-resolution split under
   `src/cli/variant/`.
2. Remove the allowlist entry when the file is under the cap.

## Keeps

- Behavior identical; a move plus wiring only.

## Proof

The ratchet passes with an empty allowlist; the system scopes stay green.

## Priority note

P3: hygiene; blocks 1304's landing only through the allowlist mechanism.

Note: number drawn from pm ticket new on 2026-10-06 and kept per the
one-sequence ruling (the draw landed in the 2000s; that is correct).
