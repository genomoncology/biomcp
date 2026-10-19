---
priority: 2
---

# 2235 — Speed up article metadata recovery

Status: OPEN.

Milestone: 0.9.4

## Outcome

Uncached `biomcp search article` keeps the approved Europe PMC metadata recovery from 2088 but spends no more than one recovery round trip on it, and PubTator visible-row metadata lookups no longer fail for rows PubTator can resolve.

## Evidence

The 2231 record (`sdlc/records/2231-uncached-article-search.md`) found uncached `search article -g BRAF --limit 10` about 1.5 to 1.8 seconds slower than v0.9.2. The metadata fallback stage grew from 719 ms to 2247 ms. Every sample had two failed PubTator metadata lookups followed by two Europe PMC recovery attempts, and the recovery also failed, so the time bought no metadata.

## Keeps

Keep the recovery source, the 60-second deadline, the row order and the truthful warnings. Change no other search stage.

## Proof

From recorded replies, show why the two PubTator lookups fail for these PMIDs: an upstream gap, or a request BioMCP shapes wrongly. Fix it if it is BioMCP's. Make the remaining recovery attempts run concurrently or as one Europe PMC request. A replay test shows the same rows, order and warnings with one recovery round trip. Five alternating live pairs show the metadata stage back near v0.9.2.

## Progress

- 2026-10-19 started
