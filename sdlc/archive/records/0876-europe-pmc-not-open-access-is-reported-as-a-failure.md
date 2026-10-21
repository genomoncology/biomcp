---
base: 9fae145f
head: 1e8a93d6
---

Europe PMC's structured HTTP-200 `errorBean` now becomes permanent absence only
when code zero and the message explicitly identify a non-open-access article.
Other XML errors, malformed XML, invalid ZIP bytes, HTTP failures, and valid ZIP
packages remain distinct outcomes.

The real receipted provider response passes through the production parser, and
all six focused supplementary-response and archive tests passed. Permanent
absence is quiet and no longer poisons a successful fallback route.
