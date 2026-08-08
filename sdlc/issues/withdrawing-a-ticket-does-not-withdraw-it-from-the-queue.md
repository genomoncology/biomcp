# Moving a ticket to drafts/ does not withdraw it from the queue

Severity: should-fix. It makes `drafts/` unsafe to rely on as a stop.

Found 2026-08-08 while arming a single-ticket shakedown flight. Thirty-four
tickets were moved from `sdlc/tickets/` into `sdlc/tickets/drafts/` and
pushed. Three cron syncs ran afterwards (15:59, 16:00, 16:01) and the queue
still held all 35 as `ready`:

    genomoncology/biomcp  ready  35

`sync-cache.json` has no entry for this repo, so a stale-cache skip is not
the explanation. Sync adds and updates rows; it does not retire a row whose
file is no longer at the top level.

## Why it matters

The documented safety property is that `drafts/` is unseen by sync. That is
true only before first import. Once a ticket has been imported, moving it to
`drafts/` — or deleting it outright — leaves it claimable, and the repo and
the queue now disagree about what the backlog is. Anyone who parks a ticket
to stop it flying will believe they have stopped it and be wrong.

It also means a ticket withdrawn because it was mistaken, unsafe, or
superseded stays dispatchable until someone edits the database by hand.

## Fix shape

- Sync knows the full set of top-level ticket files per repo on each pass.
  A row for that channel with no corresponding file should be retired —
  status `withdrawn`, not deleted, so the history and any `dep` rows
  survive and the reason stays visible in the event log.
- Retiring must not touch a ticket that is already claimed or in flight;
  those need to finish or fail on their own terms.
- `archive/` rows are exempt: they are history and have no file lifecycle.

## Workaround until then

Pause the channel. Pausing is the only reliable stop, and it is
channel-wide rather than per-ticket.
