# 2021 — WHO data gaps reach every caller

Status: complete.

Milestone: 0.9.2

## Outcome

When WHO Prequalification data is missing or partly failed, every caller sees it: Markdown, JSON and MCP. `who sync` names the failing file and column in its final error, and a partial sync does not report "synchronized".

## Evidence

Filed 2026-10-07 from the review of the work since v0.9.1 (`sdlc/issues/2026-10-07-review-of-the-work-since-0.9.1.md`, finding 12). Ticket 1304 item 4 says a validation failure must never end in a generic message; this finishes it.

- `src/entities/drug/search.rs:767-794`: with WHO data missing, the warning goes only to stderr. With the finished-product export failing on a fresh data folder, `search drug zidovudine` prints `No WHO-prequalified drugs found`, and the JSON shows `regions.who` as an empty result with no warning. With good data the same search finds 32.
- `src/sources/who_pq.rs:1099` and `src/cli/system/who_sync.rs:22`: on a header mismatch the command still exits with `Source unavailable: WHO Prequalification is not available.`, the same text as issue #288. The file and column appear only on stderr, and that line has a punctuation slip (`...BASIS OF LISTING Review source configuration and retry..`).
- A partial failure with older data kept reports `"status":"synchronized"` and exit 0 beside `"failed":["who_pq.csv"]`.

## Diagnosis

Root cause, confirmed in this worktree: the per-file failure detail that 1304
built stops at three boundaries, and every caller that is not a terminal
loses it.

1. **The search degrade signal dies at the page type.**
   `who_ready_for_region` (`src/entities/drug/search.rs`) writes the warning
to stderr and returns `(None, true)`; the bool is consumed to pick an empty
WHO page and never crosses `DrugSearchPageWithRegion`, which carries no
field for it. So the JSON envelope renders `regions.who` as a plain empty
bucket and the Markdown prints `No WHO-prequalified drugs found` — the same
bytes as a true negative. The MCP `biomcp` tool returns CLI stdout only, so
JSON and MCP callers receive no reason at all.
2. **The `who sync` final error drops its own reason.** `who_pq_sync_error`
builds a `SourceUnavailable` whose reason names the per-file outcome and
the missing files, but `BioMcpError::public_projection` collapses every
`SourceUnavailable` to `Source unavailable: WHO Prequalification is not
available.` — the generic #288 line. The file and column reach only the
stderr warning, which also carries a punctuation slip: the per-file error's
Display ends with the recovery sentence, and the warning format appends
another period (`... retry..`).
3. **A partial sync reports success.** When every failed refresh still has
an older local file, the sync returns `Ok(report)` and
`who_sync_outcome` hardcodes `"status": "synchronized"` in JSON with exit
0, beside the `failed` list.

### Success criteria

1. A region-less drug search that lost its WHO rows carries the degrade
   reason — naming WHO Prequalification and the failing export file — in the
   JSON envelope's WHO region result, in the Markdown WHO section, and in
   the MCP tool response (the same field all three read).
2. `who sync` that cannot produce the data exits nonzero with a final error
   that names the failing file and, for a validation failure, the missing
column — never the generic #288 line.
3. A partial sync says partial: JSON `status: "partial"`, per-file outcomes
   with reasons, nonzero exit; the text report never claims bare success
   beside failures.
4. 1304's spec cases (the WHO region envelope at `spec/entity/drug.md` and
   the recorded-export sync tests) stay green.

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
D
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

  P2 (temp-dir leaks in the two new spec blocks) folded in eb889a407:
  `spec/entity/drug.md` removes its temp dir via the
  `trap 'rm -rf …' EXIT` pattern from `spec/surface/cli.md`, and the
  mcp.md Python block uses `tempfile.TemporaryDirectory`. Pages re-run
  after the fold: `spec/entity/drug.md` 18/18; `spec/surface/mcp.md`
  45/46 with only the pre-existing diagnostic-synonym failure, as
  before and as already filed
  (`sdlc/issues/2026-10-07-diagnostic-synonym-provenance-fails-spec-contracts.md`).
  A review minor note folded into this record: the mcp.md degrade block
  makes three forced-degrade searches (CLI JSON, raw MCP JSON, raw MCP
  Markdown), not two.
- Fix round for the second review of the work since 0.9.1
  (`sdlc/issues/2026-10-07-second-review-of-the-work-since-0.9.1.md`),
  2026-10-08: rebased onto main (2023 and 1300 landed; the size
  inventory unioned by path with measured baselines), merged the two
  Build status sections into this one and put every review line in the
  house grammar (finding 11), and named the resolved data directory in
  the sync recovery again, with the public projection carrying the
  error's own recovery sentence (finding 17) at 168e8d866.
