# 2021 — WHO data gaps reach every caller

Status: OPEN.

Milestone: 0.9.2

## Outcome

When WHO Prequalification data is missing or partly failed, every caller sees it: Markdown, JSON and MCP. `who sync` names the failing file and column in its final error, and a partial sync does not report "synchronized".

## Evidence

Filed 2026-10-07 from the review of the work since v0.9.1 (`sdlc/issues/2026-10-07-review-of-the-work-since-0.9.1.md`, finding 10). Ticket 1304 item 4 says a validation failure must never end in a generic message; this finishes it.

- `src/entities/drug/search.rs:767-794`: with WHO data missing, the warning goes only to stderr. With the finished-product export failing on a fresh data folder, `search drug zidovudine` prints `No WHO-prequalified drugs found`, and the JSON shows `regions.who` as an empty result with no warning. With good data the same search finds 32.
- `src/sources/who_pq.rs:1099` and `src/cli/system/who_sync.rs:22`: on a header mismatch the command still exits with `Source unavailable: WHO Prequalification is not available.`, the same text as issue #288. The file and column appear only on stderr, and that line has a punctuation slip (`...BASIS OF LISTING Review source configuration and retry..`).
- A partial failure with older data kept reports `"status":"synchronized"` and exit 0 beside `"failed":["who_pq.csv"]`.

- Starts from: ticket 1304 and issue #288.
- Keeps: the header normalisation, the per-file results, and searches that degrade instead of aborting.
- Changes: carry a WHO status note in the region result for Markdown, JSON and MCP; put the failing file and missing column in the final error; report a partial sync as partial with a nonzero exit.
- Proof: outside-in tests with a recorded export missing one column: the drug search JSON carries the note, `who sync` names the column, and the partial run exits nonzero.
- Defers: nothing.

## Build status

- Built on branch `tickets/2021-who-data-gaps-reach-every-caller`,
  commits e5b98f215, e46018510, plus the review fold eb889a407,
  2026-10-07, after one timeout revival with a checkpoint (nothing
  lost).
- Code review: ACCEPT 2026-10-07. Verified: the note crosses the page
  type into the JSON envelope (clean for true negatives) and raw MCP
  verbatim; the error projection carries the sync reason and recovery
  with no doubled period; partial sync reports partial with per-file
  reasons and exit 1, older files kept; the self-referential sync hint
  is gone (network access and BIOMCP_WHO_DIR named); the failed-export
  fixture hermetic with the drift guard; inventory repins exact. One
  P2 (temp-dir leaks in the two new spec blocks) folded in eb889a407.

## Fix-round review

- Code re-review (fix-round delta 168e8d866+5796172f9): FIX 2026-10-08,
  one P1 — the error.rs baseline pinned 1446 against a measured 1447
  (the projection lifetime change grew the file one line). Everything
  else verified sound: the recovery names the resolved directory with
  the no-self-reference wording kept; the lifetime threading is
  mechanical; the ticket record parses as grammar. The P1 is fixed in
  d41d5e3ca (measured 1447, ratchet green); a fresh re-review of that
  one-line pin resolves this line.
