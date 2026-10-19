---
priority: 2
---

# 2235 — Speed up article metadata recovery

Status: OPEN.

Milestone: 0.9.4

## Outcome

- Changes: `src/entities/article/enrichment.rs` `src/entities/article/enrichment/visible_metadata.rs` `tests/test_article_metadata_recovery.py` `tests/fixtures/article/metadata-recovery/**` `CHANGELOG.md` and this ticket. Preserve generic article and variant resolution and all source-client behavior.

Uncached `biomcp search article` keeps the approved Europe PMC metadata recovery from 2088 but spends no more than one recovery round trip on it, and PubTator visible-row metadata lookups no longer fail for rows PubTator can resolve.

## Evidence

The 2231 record (`sdlc/records/2231-uncached-article-search.md`) found uncached `search article -g BRAF --limit 10` about 1.5 to 1.8 seconds slower than v0.9.2. The metadata fallback stage grew from 719 ms to 2247 ms. Every sample had two failed PubTator metadata lookups followed by two Europe PMC recovery attempts, and the recovery also failed, so the time bought no metadata.

## Keeps

Keep the recovery source, the 60-second deadline, the row order and the truthful warnings. Change no other search stage.

## Proof

From recorded replies, show why the two PubTator lookups fail for these PMIDs: an upstream gap, or a request BioMCP shapes wrongly. Fix it if it is BioMCP's. Make the remaining recovery attempts run concurrently or as one Europe PMC request. A replay test shows the same rows, order and warnings with one recovery round trip. Five alternating live pairs show the metadata stage back near v0.9.2.

## Progress

- 2026-10-19 started

## Development brief

Restated outcome: keep the approved visible-row metadata recovery while eliminating duplicate Europe PMC lookups and running the remaining recovery requests in one concurrent wave, under the existing deadline. Preserve row order and truthful warnings.

Study jobs `2235/1792418910837-57856` and `2235/1792418942048-76361` captured the exact PubTator and Europe PMC requests for PMIDs 42891779 and 42894377. PubTator returns 400 with Could not retrieve publications; exact Europe PMC requests return 200 with hitCount zero. The PubTator request shape matches existing successful captures. This supports an upstream coverage gap; it demonstrates no BioMCP request-shaping defect and does not prove all future upstream failures have that cause. The first job's resultType=core Europe PMC captures are auxiliary; use the second job's exact client requests for regression fixtures.

Source inspection found that the generic article resolver already tries Europe PMC after PubTator 400, then visible-row enrichment repeats that recovery after the resolver returns an error. Two rows can incur four sequential Europe PMC requests. Introduce a helper confined to plain visible-row metadata; leave generic detail and variant callers unchanged. Collect PubTator outcomes, run one Europe PMC request per failed row concurrently, and apply results and warnings in original row order.

Write the outside-in CLI regression first using synthetic search scaffolding explicitly labeled as such and the genuine captured recovery replies. On current main it must show repeated recovery requests; require exactly one per PMID and overlapping recovery requests through a barrier, without an elapsed-time threshold. Also exercise successful recovery. Record exact commands and red/green outputs. Run only affected checks, then five alternating live comparison pairs using the existing measurement method on the configured build host. Preserve invocation flags, metadata-stage timings, source identities, load and limits. Escalate if matching the requested comparison requires a product decision. Push source once ready for review.

Study raw replies and receipts remain under `.cache/2235-study` in the job checkout. Retain a durable evidence archive before removing the only copies. PubTator body SHA256: `922593f698154883679e4a5f7e7d10c699238d74cb3206e18c47f421588816ba`; exact Europe PMC body hashes: `0d721f00c39788a3666061741263248e2563e125b0f8237cd5669c7b81eecec8` and `4fb83edf13f854924aae95801e680c243a7f4312b392bd55e4dc0530d8b1e93b`.
