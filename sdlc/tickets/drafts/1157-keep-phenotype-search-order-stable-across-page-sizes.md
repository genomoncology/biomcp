---
flow: build
priority: 7
---

# Keep phenotype search order stable across page sizes

## Goal

One phenotype query returns one stable result order across supported page sizes and offsets. On 2026-09-04, direct Monarch requests for `HP:0000256` returned different first diseases with limits of two and three. BioMCP derives the provider limit from the requested output window and then pages the changing candidate set locally. The reproduction and code path appear in `sdlc/issues/2026-09-04-phenotype-result-order-changes-with-limit.md`.

## Desired functionality

BioMCP obtains a stable supported candidate window for a normalized phenotype query before it applies the requested limit and offset. Successive pages preserve the same order as one request for the combined window. Output identifies any provider coverage ceiling.

## Success criteria

- Results from the fixed provider fixture have the same prefix for limits of one, two, three, and five.
- Two adjacent pages contain the same ordered rows as one request for their combined range.
- Paging does not repeat or skip a disease inside the supported window.
- Human-readable, JSON, and MCP output identify a provider coverage ceiling when one applies.
- A fixed provider fixture proves the behavior without a live request.

## Boundaries

This ticket stabilizes pagination for one normalized phenotype query. It does not change the provider's similarity score, promise complete coverage beyond the supported window, or determine whether a candidate directly carries the phenotype.
