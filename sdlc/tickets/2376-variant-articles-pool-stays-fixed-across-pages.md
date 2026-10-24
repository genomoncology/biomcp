# 2376 — Variant articles pages over a fixed pool

Status: OPEN.

Milestone: 0.9.4

Priority: 0

## Outcome

Paging `variant articles` walks one fixed candidate pool: each PMID appears on exactly one page, and paging to the end returns the whole pool every time.

## Evidence

- Research test of the 0.9.4 practice artifact (run 39857815354, SHA-256 6dbc3a83...4739) on Blue, 2026-10-23, relayed by Jarvis in m4147. Plain PubMed ESearch for 'AKT1 E17K' (relevance sort) returns 192 PMIDs. `biomcp --json variant articles 'AKT1 E17K'` (union, --limit 50, 35 pages) returns about 1,386 PMIDs, yet 20 of PubMed's 192 never appear. Ian's ruling: the worst case is variant articles failing to return what a plain PubMed search finds.
- Union's reported candidate pool changed between pages during the run. The 2006 AKT mutations paper appeared near rank 390 in one run and was missing from another, longer run. Related: 2365 found CHEK2 G167R paging returning 0 rows with has_more=true.

## Proof

- Outside-in replay test: paging AKT1 E17K to the end twice gives the same ordered PMID list with no duplicates and no gaps, and the pool size is the same on every page.
- Lands before the final 0.9.4 requalification and rehearsal; research rechecks AKT1 E17K on the final build.

## PM ruling on continuation (2026-10-23, answering m4164)

Approved: an optional `--page-token` CLI argument and a matching typed MCP input, carried by the existing `pagination.next_page_token` output and bound as the design states. A token always reads its own snapshot until expiry; an expired token fails explicitly and tells the caller to restart from offset 0.

Successor sessions: a call at offset 0 without a token starts a session. It selects the newest snapshot for its query identity, building or resuming one if needed, and makes that snapshot the active one for legacy offsets. Nothing else switches the active snapshot. Retries and late rows publish successors but never change the active snapshot by themselves.

Legacy offsets: a nonzero offset without a token reads the active snapshot. If none exists or it expired, it fails explicitly and asks for a restart at offset 0; it never rebuilds at a nonzero offset. Two legacy walkers of the same query share the active snapshot, so a second walker's offset-0 call can switch the first walker's pool when a successor exists. The help and docs say so and point concurrent callers to the token. Every response carries a token, so MCP clients and agents get the stable path by default.

## Changes

The shared article path now acquires, merges and ranks candidates before it publishes a versioned immutable pool in the existing cache. It retains native provider progress separately and publishes late results as successors. Only an offset-zero call without a token activates a pool for legacy offsets. Nonzero offsets and bound continuations fail explicitly when their pool is unavailable; they never rediscover a replacement.

CLI `--page-token` and each typed MCP batch item's `page_token` use the same continuation implementation. The token binds query identity, source configuration, genome assembly, verification options and ordering version. Cache bypass still supports offset zero; it cannot issue a persisted continuation or serve later offsets. The help and variant/MCP guides explain this boundary and the shared active pool for concurrent legacy walkers.

The private stored representation preserves captured article fields omitted from public JSON, full identity observations and linkage, canonical aliases, provenance and original debug construction facts. Cache decoding rejects unknown versions, atoms and invalid identity state. Page verification can annotate newly visible rows without changing frozen order. Provider admission has separate clocks and permits under the existing invocation ceiling; retained rows do not consume the next invocation's native acquisition allowance.

Outside-in cases cover token and legacy walks, per-item MCP pools, successors, refused option changes, expiry and corrupt stored state, retained source tails, and identity/provenance round trips. Existing source-status and spec continuations now establish a retained offset-zero session before requesting later offsets.
