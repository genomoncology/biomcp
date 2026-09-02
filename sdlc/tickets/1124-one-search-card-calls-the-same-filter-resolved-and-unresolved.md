---
flow: build
priority: 3
---

# One search card calls the same filter resolved and unresolved

`biomcp search variant -g RB1 --hgvsp Q999X` prints both of these in a single card, on `0.9.0-dev.6`, verified 2026-09-02:

```
Resolution: Unresolved

## Filter resolution
gene: resolved
hgvsp: resolved
```

The two lines answer different questions. `Resolution:` asks whether the requested identifier named a known variant. The `## Filter resolution` block, added by ticket 1094, asks whether each filter value was recognised and submitted to the provider. A reader has no way to learn that from the output, so the card appears to contradict itself about `hgvsp`.

Ticket 1094 created this. It asked for a per-filter signal and did not say to reconcile the new block with the line already there. Same class as ticket 1029: use one vocabulary for a section's outcome.

## Required behavior

A single card never applies the same word to the same filter with two meanings.

Either the two outcomes use distinct words, or the older line states which question it answers. A reader who sees both lines can tell what each one is about without reading the source.

## Done, observably

- `search variant -g RB1 --hgvsp Q999X` reads coherently. No word describes `hgvsp` twice with opposite senses.
- `search variant -g H3F3A --hgvsp K28M`, which resolves, still reports resolution and still reports its filters.
- A gene-only search, which has no identifier to resolve, prints no contradictory pair.
- The `--json` shape carries the same distinction the Markdown does.

## Boundary

Do not remove either signal. Both carry information a caller uses.

Do not change which filters are submitted, which records are returned, or the results table.
