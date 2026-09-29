# The README understates the source count and misses cell-line

Filed 2026-09-28 by the marketing lead while counting BioMCP's entities and sources for a talk slide. Checked at main `9f016069`.

## What is wrong

1. `README.md` line 7 says BioMCP reaches "~30 trusted biomedical sources". `docs/reference/sources.json` lists 78 sources, and BioMCP calls 70 of them directly. The other 8 appear only inside another source's answer.
2. The README's table of `get` entities lists 12. `src/cli/list/catalog.rs` lines 38 to 54 hold 15 entities, and 13 of them work with `get`. The table misses `cell-line`.

## Evidence

- `biomcp list` from a main build prints the same 15 entities as `catalog.rs`.
- Release v0.9.0 had 14 entities, without cell-line, and 68 direct sources out of 76.

## What marketing needs

The README states the counts from `sources.json` and `catalog.rs`, or derives them, so they cannot drift again. Marketing will quote 15 entities and 70 sources.

## Resolved 2026-09-28

The README now states the counts the registry and catalog carry —
70 direct sources of 78 total (from `docs/reference/sources.json`,
verified by command the same day) — and the gettable table lists
cell-line with its Cellosaurus providers and an example. The counts
are stated, not derived; the source registry's own contract
(`tests/test_source_licensing_docs_contract.py`) pins the registry
side, and `biomcp list` prints the catalog side, so a drift shows in
one command. Marketing can quote 15 entities and 70 sources.
