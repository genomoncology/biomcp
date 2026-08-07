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
The ready/blocked/draft tickets (666, 671, 673–677, 682, 684, 686,
688, 499) become real sdlc tickets — but in `sdlc/tickets/drafts/`,
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
