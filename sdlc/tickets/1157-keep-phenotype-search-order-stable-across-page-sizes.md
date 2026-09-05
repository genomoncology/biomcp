---
flow: build
priority: 7
---

# Keep phenotype search order stable across page sizes

## Goal

One phenotype query returns one stable result order across supported page sizes and offsets. On 2026-09-04, direct Monarch requests for `HP:0000256` returned different first diseases with limits of two and three. BioMCP derives the provider limit from the requested output window and then pages the changing candidate set locally. The reproduction and code path appear in `sdlc/issues/2026-09-04-phenotype-result-order-changes-with-limit.md` at commit `62575a99`.

## Desired functionality

For every normalized phenotype query, BioMCP makes one bounded Monarch similarity request with `limit=50`, retains the provider's fixed order, normalizes and deduplicates the complete candidate window by disease ID before slicing, and keeps the highest-ranked (first provider-ordered) occurrence. Requested limit and offset are then applied locally. Tied scores retain provider order rather than acquiring a new unstable secondary sort.

The provider boundary preserves the raw response row count before MONDO filtering or deduplication. Pagination exposes `provider_window_limit: 50` and a separate provider-window exhaustion boolean that is true whenever the raw response contains exactly 50 rows. `has_more` means only that another locally buffered normalized row exists. Both may be true: continuation remains available while buffered rows remain, and the final supported page omits continuation while warning that additional provider matches may exist beyond the 50-row ceiling.

## Success criteria

- A fail-closed request log proves limits one, two, three, and five and their supported offsets always produce exactly one normalized-query Monarch request with `limit=50`; former limit-dependent provider shapes (`2`, `3`, `4`, and `6`) are rejected.
- An adversarial fixture makes the old limit-dependent routes return incompatible orderings. Limits one, two, three, and five are prefixes of the same fixed 50-row response, and page `(offset=0, limit=2)` plus page `(offset=2, limit=3)` equals `(offset=0, limit=5)`.
- Duplicate disease IDs in the raw window are deduplicated before slicing, retaining the first/highest-ranked provider occurrence. Local page boundaries neither repeat nor omit normalized rows from that fixed sequence; tied scores retain provider order.
- The provider client carries raw row count/exhaustion metadata before MONDO filtering and deduplication. A short raw response reports `provider_window_limit: 50` without exhaustion; an exactly-50-row response reports exhaustion even when normalization removes rows.
- A state with more buffered rows and provider exhaustion exposes both `has_more: true` and the exhaustion boolean, offers the next local offset, and warns about the ceiling. The final buffered page has no continuation but retains the warning that additional matches may exist.
- CLI Markdown, CLI JSON, and raw MCP `biomcp` calls in both default and `json:true` modes agree on continuation and provider-window metadata. This ticket does not add phenotype to the typed MCP `search` schema.
- Existing `offset + limit <= 50` validation still fails before provider contact. Monarch transport, non-success status, content-type, and decode failures remain typed provider errors rather than partial or empty success.
- Fixed provider fixtures prove all behavior without a live request.

## Boundaries

This ticket stabilizes bounded pagination for one normalized phenotype query and makes the provider ceiling truthful. It does not change the provider's similarity score, promise complete coverage beyond the 50-row window, determine whether a candidate directly carries the phenotype, make more than one provider request per query, or add a typed MCP phenotype-search surface.
