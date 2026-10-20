---
base: 65e140de
head: 3f6916db
---

Rebuilt the five phrase-banning checks as structural checks, from
Ian's feedback: structure closes the whole class; more banned
phrases never will.

The review-status contract now enforces one Review grammar over
every ticket file: logical lines join wrapped continuations before
matching (a wrapped "re-review pending" cannot pass), a REJECT or
BLOCK needs a later ACCEPT of the same kind or an inline resolution
word, and a pending scope may only name a slice the ticket itself
defines — the scope's numeric tokens must appear in the ticket's
own body, so "(batch 41)" on a ticket that never had one fails.
Legacy tickets 1191-1218 carry backfilled verdicts taken from
their records and git history, with "record not kept at the time"
where history is silent — no invented verdicts (reviewer
spot-checked seven). 1249's REJECT resolves inline as its record
honestly admits: no separate re-review accept exists; the fix was
confirmed and pinned. Eight never-told-about mutations go red.

The workflow contract hash-pins 19 load-bearing steps: each pinned
step is extracted from the parsed workflow YAML, whitespace
normalized, and compared by SHA-256 recorded in the test; any edit
to a pinned step — including `if:` conditions, shells, and
continue-on-error removals — changes the hash and fails with the
step named. Mutations prove one-character drift and all six leak
spellings from the review file red, including `runner.os ==
'Linux'` on the smoke step, which only the hash catches. `shell:`
outside a step fails structurally at both defaults levels.

The wait ratchet counts its markers (24, ceiling 24 — raising the
ceiling takes a deliberate inventory edit) and catches the forms
the old patterns missed: any `elapsed()`-comparison wait,
`sleep_until(`, `anyio.sleep`, and aliased time imports resolved
by local name. The stdio guard resolves use-import aliases
(`Command as Cmd`, braced and bare) and checks every call site of
the resolved name; unknown process-module renames fail closed.
`merge_property` panics on any one-sided constraint keyword
(uniqueItems, maxLength, ...) naming the keyword, with should-panic
tests; the one real collision (typed_get's `sections`
uniqueItems) was resolved explicitly — duplicate rejection stays
body-side — and ADR 0002 now states the precise rule: the merged
root never accepts less than any one branch.

Evidence: code review BLOCK once — two P0s the no-cargo review
could still see (a dead old-code block breaking ruff; a fmt
indent) plus the missing scope-vocabulary check — all folded with
the reviewer's exact payloads (folding itself surfaced a leaked
loop variable the tests then caught). Gate cycles caught a
serde_json key-deref mismatch, the ruff findings, and one
load-flake of the stdio timing test (passes four times alone);
green at 3f6916db — lint, test, spec, stress, zero failed lines.
