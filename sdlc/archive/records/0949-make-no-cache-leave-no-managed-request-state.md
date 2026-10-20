---
base: 8014e0927199a6abc8f46cb6501d841a241bc920
head: 6b7bff435714adff7edcd3c9e07b7684ab431394
---

`--no-cache` requests bypass managed HTTP-cache initialization, reads, writes,
and cleanup, including success and failure paths. Focused filesystem contracts
prove that a no-cache command leaves no managed request state behind.
