# Final live replay qualification

Job 2258/1792451039742-39317 passed at pushed source 51f5e03b218c87cedc82bd9e1a67552bd5ab9df8 using the existing checker once:

```sh
python3 tools/check-replay-live --live --disable-sources commercial-restricted,semantic-scholar
```

| Total | Checked | Passed | Disabled | Blocked | Drift | Ready |
| --- | --- | --- | --- | --- | --- | --- |
| 47 | 39 | 39 | 8 | 0 | 0 | 0 |

The complete result has 47 entry rows plus its summary. Exit status is zero; stderr is empty. This qualifies the enabled recordings for response status, media type and shape at this source revision and observation window. It does not establish clinical validity, equal values or future provider stability. The checker permits added paths; recordings 8, 10, 18, 19, 34, 35, 36, 37, 38, 39 and 42 reported additions without losing required recorded structure.

## Explicit unverified entries

| Source | Recording numbers | Disposition |
| --- | --- | --- |
| KEGG | 14, 15, 16 | not checked live: commercial-restricted source |
| Semantic Scholar | 40, 41, 45, 46, 47 | not checked live: rate limited |

Neither source received a request in this qualification. These eight entries count only as disabled, never checked or passed. The Semantic Scholar disposition follows the PM ruling after the one later 429 diagnostic in record 2280. No new key, privileged access, cost or provider contact was used. The commercial-restricted selector also disables the corresponding registry IDs; only KEGG contributes affected recordings in this inventory.

## Preparation and evidence

Preparation job 2258/1792450988450-21123 stopped before any checker invocation or provider call because the isolated system Python 3.9 lacked tomllib. The corrected invocation uses the already installed Python 3.12.13. This was the only actual final live-check invocation. An empty home and explicit minimal environment inherited no credentials, proxies or source overrides. Default certificate verification, existing serial pacing, size caps, redirect refusal, source restrictions and retry policy remained intact.

Private experiment 2102-biomcp-2258-final-live-replay, owned by biomcp/2258 and marked Keep, retains checker.jsonl, qualification-job.log, empty checker.stderr, inventory.json, settings.json, result.json, report.txt, source-sha.txt, exit-code.txt and the failed preparation log. The unchanged checker emits no raw response status values, Retry-After values, request timestamps or provider bodies; this record does not invent that missing wire evidence. Earlier primary raw observations and repairs remain separately in experiments 2096 through 2101.

This live result completes one admission check. It does not authorize a release tag or publication and does not replace offline, measurement, full-main, review or full rehearsal evidence.
