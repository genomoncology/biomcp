# Onboarding BioMCP to the sdlc factory — the prep work

Written 2026-08-07 by the sdlc side. This document is the handoff:
everything the BioMCP team can do now, how to know each piece is
done correctly, and why none of it can accidentally start the
machine.

## Safety first: why nothing here triggers anything

The factory only ever looks at repos listed in its registry on
beelink (`~/.local/share/sdlc/repos`). BioMCP is not in it. Until
someone deliberately registers this repo — one command, run by Ian
or his workspace agent, as the very last step — every file under
`sdlc/` here is inert: tickets, scripts, all of it. Create freely.
As extra belt-and-braces, tickets that exist but should not run yet
can sit in `sdlc/tickets/drafts/`, which the factory never reads
even after registration.

## The prep work, in order, each with its "done" check

**1. The `sdlc/` folder and scripts.**
Copy `~/workspace/repos/sdlc/template/sdlc/` into this repo. Adapt
the tail of `scripts/lint` and `scripts/test` to this project's
real commands (the Makefile targets March used), and
`scripts/prepare` to install dependencies into a fresh worktree.
Do not modify the plumbing at the top of the scripts.
*Done when:* `sh <this-repo>/sdlc/scripts/lint` and `.../test` both
exit green **run from an unrelated directory** (for example from
`$HOME`). The dispatcher never runs them from the repo root, so
that is the test that counts.

**2. Convert the completed March tickets into archive history.**
Target: one file per completed ticket in `sdlc/tickets/archive/`,
named `NNNN-<original-slug>.md` (March's number zero-padded to four
digits, slug unchanged), containing frontmatter
(`flow: build`, `priority: 5`), the ticket's title, a few summary
lines, and pointers to `~/workspace/planning/biomcp/artifacts/<slug>/`.
Where recoverable, a matching record in `sdlc/records/` carries the
landed commit range as `base:`/`head:` frontmatter. The sources and
the recovery recipes (ticket → runs → sessions → commits) are
documented in `~/workspace/repos/deck/sdlc/planning/march-harvest.md`;
the ticket index is `~/workspace/planning/biomcp/tickets.json`
(689 entries). Write this as a script, not by hand — it will run
again for other teams.
*Done when:* the script prints a coverage report — how many tickets
converted, how many commit ranges recovered, how many could not
be — and a spot check of five random archive files against their
artifact folders holds up. Anything unrecoverable is omitted from
the record, never guessed.

**3. Rewrite the live March tickets.**
The ready/blocked/draft tickets (666, 669, 671, 672, 673–677, 682,
684, 686, 688–690, 499 — sixteen; 669/672 were missed in this
document's first draft, 689/690 postdate it) become real sdlc
tickets — but in `sdlc/tickets/drafts/`,
not the top level. Follow the `/ticket` skill (available in any
workspace session): four-digit id keeping the March number, one
behavior per ticket, frontmatter, plain language, a raise allowance
if code will grow. A March ticket that bundled several behaviors
becomes several tickets.
*Done when:* each draft stands alone — a reader with only the
ticket text and this repo could start work without asking a
question. That is the bar; Ian or the workspace agent will read
them against it before promotion.

**4. Triage the open March issues.**
The 21 open issues move to `sdlc/issues/`, one short file each with
file/line where known and severity (blocking / should-fix / minor).
Stale ones are dropped — named in the commit message, not silently.
*Done when:* `sdlc/issues/` matches reality and the commit history
says what was dropped and why.

## What happens after (not this team's work)

Ian's side lands the remaining queue safety tickets, verifies the
prep, promotes drafts in chosen order, and registers the repo. From
that minute the factory dispatches BioMCP tickets one at a time.
New ticket numbering continues from 0689.

## Skills available in any workspace session

`/onboard` (this process in general), `/ticket` (the format and
bar), `/triage` (issues into tickets), `/lead` (planning
conventions). They live in `~/workspace/repos/sdlc/skills/` and are
symlinked workspace-wide.

## Answers to the team's seven questions (2026-08-07, Ian + sdlc side)

1. **Priority: do NOT flip the numbers.** The sdlc claim query is
   `ORDER BY priority DESC` — higher runs sooner, same as March.
   The ticket skill briefly said the opposite; that was wrong and
   is fixed. Carry March's numbers straight across.
2. **The live list was short by four** — 669 and 672 were missed
   (not deliberate), 689/690 postdate the doc. Corrected above:
   sixteen tickets.
3. **Superseded: skip them, as you recommended.** No archive entry
   for work that never landed. But the conversion script's
   coverage report must count them ("skipped: 118 superseded") so
   the omission is visible, never silent.
4. **The ten-minute prepare: leave it.** Correct and slow beats
   fast and porous, and the ticket rate is unknown. "Cache the
   green" is noted as a future sdlc-template feature; it is not
   BioMCP's problem to solve.
5. **`make spec` maps to the spec rung; `make verify` joins no
   rung.** The live-credential lane is not retired — it becomes an
   operator-run (or separately scheduled) check outside the
   factory, because gating unattended tickets on live external
   services makes flights fail for reasons no agent caused. Note
   its existence and how to run it in `sdlc/planning/`.
6. **Move the 18 actually-open issues; leave converted and closed
   as March history.** The "21" came from March's status output;
   the files are the truth.
7. **Confirmed: you do all four prep jobs directly in this repo.**
   The no-hand-edit rule was March-era discipline and retires with
   March. The sdlc side verifies against the done-checks and runs
   the registration. The March queue stays paused permanently — it
   is never resumed, not even to land remaining work; everything
   still open lands through sdlc or not at all. And agreed on 499:
   it stays a draft until a publish flow exists.
