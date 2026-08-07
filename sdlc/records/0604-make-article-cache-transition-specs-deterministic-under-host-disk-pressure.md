---
base: 972b15246ab970f306ff3064fef613a81c183efe
head: 428a3a68e1d224b43258a706a8b7b175fd241d8c
---
Ticket 599 added a routine article-full-text contract that makes two separate BioMCP invocations against the same cacheable fixture URL: the first must report `miss`, and the second must report `hit`. The fixture inherits BioMCP's production cache floor of 10% free disk. When the host is below that floor, the first process schedules an asynchronous cache eviction; depending on scheduling, the newly written fixture entry is deleted before the second process reads it.

Imported from March ticket 604. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/604-make-article-cache-transition-specs-deterministic-under-host-disk-pressure
