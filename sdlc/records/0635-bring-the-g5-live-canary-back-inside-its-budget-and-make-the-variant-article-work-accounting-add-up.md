---
base: 12136b34405fd7f1810215f4d5d8ed4b60499027
head: 0be2832c79ad20014832624f7aecd999a0d04421
---
Ticket 634 un-starved the exact route, which more than doubled the G5 canary's runtime past its 180s budget; the exact pool is also too small to finish, so every live result is incomplete, and the debug plan's work accounting contradicts its own recorded calls.

Imported from March ticket 635. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/635-bring-the-g5-live-canary-back-inside-its-budget-and-make-the-variant-article-work-accounting-add-up
