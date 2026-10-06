# 2010 — Split the system dispatch CLI file

Status: OPEN.
Milestone: 0.9.2

## Outcome

`src/cli/system/dispatch.rs` returns under the 700-line CLI cap and its
allowlist entry retires.

## Evidence

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
