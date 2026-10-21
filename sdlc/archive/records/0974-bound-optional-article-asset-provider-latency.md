---
base: 61474ad5
head: ae27d96d
---

Article-asset discovery now gives the primary PMC archive an overall 30-second
budget and each optional provider rung an overall 10-second budget, including
its internal retries and waits. Available assets survive an optional timeout.
The manifest records data, degraded data, healthy absence, source failure, and
timeout as distinct provider outcomes.

Normal manifest requests reuse a private, bounded cache entry for five minutes,
so paging does not immediately repeat a degraded provider ladder. Expired,
oversized, malformed, linked, or wrong-schema entries are rejected; `--no-cache`
bypasses reads and writes. Raw asset retrieval remains live because cached
manifests never store asset bytes.

Focused tests covered deadline settlement, stable outcome serialization, fresh
reuse, expiry, corruption, and private atomic cache writes without public
network access or long wall-clock sleeps.
