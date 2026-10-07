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
