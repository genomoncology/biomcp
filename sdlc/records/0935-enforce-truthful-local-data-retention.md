---
base: 8014e0927199a6abc8f46cb6501d841a241bc920
head: eb11d837082e8aa2920aae83675d2f6dc19ba7db
---

Managed HTTP cache entries and sessions now expire according to their stated
retention periods. Reads, listings, statistics, and routine startup remove
expired state consistently, with injectable clocks and focused boundary tests
covering the retention rules.
