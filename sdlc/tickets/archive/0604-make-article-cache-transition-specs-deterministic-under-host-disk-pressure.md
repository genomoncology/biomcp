---
flow: quickfix
priority: 10
---
# Make article cache-transition specs deterministic under host disk pressure

Ticket 599 added a routine article-full-text contract that makes two separate BioMCP invocations against the same cacheable fixture URL: the first must report `miss`, and the second must report `hit`. The fixture inherits BioMCP's production cache floor of 10% free disk. When the host is below that floor, the first process schedules an asynchronous cache eviction; depending on scheduling, the newly written fixture entry is deleted before the second process reads it.

Completed under March on 2026-07-20, as March ticket 604. Imported as history when BioMCP
moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/604-make-article-cache-transition-specs-deterministic-under-host-disk-pressure
