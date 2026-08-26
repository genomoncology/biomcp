---
flow: build
priority: 10
---

# Make the documented FAERS --count invocation work

Found in the 2026-08-26 slide-lab run (experiments/184, keyless, binary
0.9.0-dev.6), re-verified against current main the same day.

## What a user hits

`biomcp search adverse-event --drug pembrolizumab --count reaction` — the
invocation the help line itself suggests — fails:

    Error: Invalid argument: --count requires explicit --source faers

The `biomcp list adverse-event` help line for `--count <field>` describes
"OpenFDA FAERS aggregation" and enumerates the fields, but never states
that `--source faers` must accompany `--count`. Every first-time user of
the aggregation surface hits this error; the slide-lab run did.

## The design choice this ticket settles

Two legitimate repairs, and the design stage picks one and pins it:

- Make `--count` imply the faers source — aggregation is only implemented
  against OpenFDA FAERS today, so the flag combination is unambiguous —
  and the documented invocation simply works.
- Or keep the explicit requirement, state it in the `--count` help line,
  and make the error text itself carry the fix ("add --source faers").

Either is acceptable; silently changing what `--source vaers --count`
means is not — if vaers aggregation is requested, it must still fail
loudly with a truthful reason.

## Done when

- The documented invocation (`--count` without `--source`) either succeeds
  against FAERS or the help line names the requirement before the user can
  hit the error.
- The chosen behavior is pinned by contract tests: the working invocation
  (if implied) or the help-line text (if documented).
- `--source vaers --count` still refuses with a truthful message, pinned
  by a test.
- The `biomcp list adverse-event` help text matches the chosen behavior.

Filed as build, not quickfix: the suite is green — no existing assertion
reproduces a fault, the gap is between documented usage and a required
flag, and the proof must be authored (the 1050 refusal this week is the
precedent: quickfix refuses a green suite).
