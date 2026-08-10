# Seven clinical tickets have no owner and nothing holds them

The 2026-08-09 adversarial review found eight biomcp tickets *"structurally
usable, but clinical/genomics content requires a qualified owner."* The
2026-08-09 handoff then recorded "domain owner for the 8 biomcp clinical
tickets" as **blocking channel resume**.

Neither of those is true of the queue. Seven of them are `ready` right now:

| ticket | priority | subject |
|---|---|---|
| 0874 | **9** | protein annotation can come from a different transcript |
| 0878 | 5 | gnomAD v4 and filtering allele frequency |
| 0879 | 5 | gnomAD quality filter flags |
| 0688 | 4 | broaden the dual-build collision corpus to 41 coordinates |
| 0881 | 4 | ClinGen expert panel assertions |
| 0882 | 3 | full-text tables are dropped |
| 0883 | 3 | reading cached full text without flooding context |

(The eighth, 0689, has been rescoped and now waits on 0898.)

**0874 is priority 9 — second in the claim order.** Resume the channel and it
flies unattended, on a clinical question, with no owner named.

## The real problem underneath

The gate was recorded in a handoff note. A handoff is read by the next
session, not by the dispatcher and not by the agent that picks the ticket up.
Nothing in the ticket bodies mentions it, so nothing enforces it.

There is also no supported way to hold specific tickets. `drafts/` does not
work after import — sync never retires a row whose file is gone, so a drafted
ticket stays `ready` with a `ref` pointing at a path that no longer exists.
Pause is channel-wide. Priority only reorders. This is the capability Ian
asked for on 2026-08-08 and it is filed as
`repos/queue/sdlc/issues/an-imported-ticket-must-still-be-controllable.md`.

## Options

1. **Rule that they may fly.** The review called them structurally usable;
    the concern is whether the clinical content is *right*, which design,
    design review, and a later human read would each catch. Cheapest, and
    defensible for the lower-priority ones.
2. **Write the gate into each ticket body** — "do not build this until a
    qualified clinical owner has signed off; refuse if you reach this stage
    without that recorded." Supported today, uses only the ticket body, and
    an agent will stop. Costs one refused flight per ticket to discover it,
    which is wasteful but safe.
3. **Drop them to priority 0** so they sort behind everything. Delays rather
    than holds, but buys time without touching the bodies.
4. **Name an owner** and let them fly with that person reviewing the landing.

Recommend 1 or 4 for the low-priority ones and an explicit decision on
**0874** before the next resume, because that is the one that will actually
be claimed.

## Ask

This is a domain call, not an engineering one. Raised here rather than left
in a handoff so it survives the session that wrote it.
