---
base: e5ef3bb2968bb859ef4d6f43d0b7da2593921178
head: 205768a5452fb00ee93beeac00aacfcfbb7f8721
---

BioMCP now carries the canonical transient fetch recovery in its lifecycle
`tasks` script. A failed fetch receives one retry, and repeated failure reports
a sanitized, bounded final diagnostic.

Consumer contracts preserve this behavior so dispatch can resume without
hiding useful fetch failures.
