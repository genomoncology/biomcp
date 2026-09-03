# Retiring or superseding a ticket without doing its work

Written 2026-09-03 by the BioMCP lead, after retiring ticket 1118 and superseding ticket 1132 on the same day.

## The rule

A ticket whose work will never be done goes to `sdlc/tickets/drafts/` with a `hold:` line saying why, and a banner at the top of the body saying the same thing in prose.

It does **not** go to `sdlc/tickets/archive/`.

## Why

`sdlc/project/tasks` reports status from the tree, and its rule is written in its own header: status is `done` for a ticket with a completion record carrying the same id, **and for anything under `sdlc/tickets/archive/`**. There is no third state. A ticket is `draft`, `ready` or `done`.

So archiving a ticket nobody worked tells the board the work happened. Verified on 2026-09-03: `factory ticket botassembly/biomcp/1118` reported `state: done` with `attempts consumed: 0` and `work dispatched: 0`. Zero attempts and done at once, which is the shape of the lie.

Ticket 1081 is the same mistake sitting on the record from earlier. It was archived when ticket 1123 took over the same work, and it has counted toward this channel's done total ever since.

A held draft is honest instead. `tasks` reports it as `draft`, drafts are never dispatchable, and the `hold:` line is a judgment hold only a person releases.

## What archive is for

Work that landed. A ticket whose record exists, or whose behavior shipped by another route and can be shown to have shipped.

## The cost, stated

A retired ticket stays visible on the board as a draft forever, which is a small amount of permanent noise. The alternative is a false done, and a wrong count is worse than a visible parked file. Deleting the file instead would read as `withdrawn`, which is honest about the state but throws away the written reason, and the reason is the only durable thing a retired ticket has.

## Overturning this

Cheap. Move the file to `sdlc/tickets/archive/` and delete the `hold:` line. Two retired tickets carry the pointer to this note and would need their banners edited.

The better fix belongs upstream: the factory has no ticket-level retired state, so every repository making this call has to pick between a false `done` and permanent draft noise. That is a platform gap, not a BioMCP one, and it is recorded as feedback rather than acted on here.
