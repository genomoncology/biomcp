---
base: e94e5b61
head: 6d2ecbee
---

Article asset lookup now distinguishes a known nonretrievable NCBI asset from
an unknown asset. The typed error carries only opaque lookup keys, gives the
safe PMC browser URL, and never exposes provider interstitial content or an
invented download URL.

The routine local fixture covers both key and filename lookup, the browser
fallback, and the unchanged unknown-key not-found result. Focused asset tests
and the owning specification passed.
