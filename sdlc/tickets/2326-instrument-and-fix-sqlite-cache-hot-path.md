# 2326 — Make warm cache reads as fast as 0.9.3

Status: OPEN.

Milestone: 0.9.4

## Outcome

Warm and day-cached answers return within 0.5 ms of 0.9.3 on the ticket 2234 rows, measured on the same Mac, while the cache keeps its integrity checks, safe use by concurrent processes, and bounded size. Measure first: add timing spans for Store::open, startup maintenance, transaction begin, the read and close, plus a busy-handler call counter, and attribute the 5.08 ms warm get variant and 1.16 ms day-cached search gwas gaps before changing behavior. Then change one step at a time, measuring after each: read in deferred transactions with blob re-validation and one retry, keep one connection per process, and skip startup maintenance when nothing is due. Move pressure eviction and last_access writes off the request path, and evict in batches. Keep SQLite and the current schema.

- Proof: paired 0.9.3-versus-candidate rows from the 2234 method on the same Mac are within 0.5 ms; a stress test with an evictor in one process and readers in another returns only correct bodies or misses; the attribution spans show where the original gap went.

## Evidence

- sdlc/records/2234-candidate-measurements.md measured median paired increases of 5.078499 ms for warm get variant and 1.164458 ms for day-cached search gwas, with the cause unknown. 0.9.4 accepted them as known costs.
- A 2026-10-21 design review of origin/main 7efd8930d ranked the suspected causes from code reading:
  - Every get runs in an IMMEDIATE transaction (src/cache/sqlite.rs:130-133). Concurrent gets from joined requests then sleep in SQLite's busy handler (busy_timeout 100 ms, sqlite.rs:93) for 1 ms or more.
  - Every get and put opens a fresh connection (src/cache/manager.rs:143, 233), which creates and removes WAL files each time.
  - Startup runs two write transactions even when nothing is due (manager.rs:80-91, sqlite.rs:355-357, 404-405).
- The same review found SQLite's overhead fixable and recommended against switching stores.
- Ian asked on 2026-10-21 whether eviction can run separately, whether faster bulk deletion exists, and for a better long-term design.
