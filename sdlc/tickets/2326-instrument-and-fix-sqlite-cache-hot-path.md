# 2326 — Make warm cache reads as fast as 0.9.3

Status: OPEN.

Milestone: 0.9.5

## Outcome

Warm and day-cached answers return within 0.5 ms of 0.9.3 on the ticket 2234 rows, measured on the same Mac, while the cache keeps its integrity checks, safe use by concurrent processes, and bounded size. Measure first: add timing spans for Store::open, startup maintenance, transaction begin, the read and close, plus a busy-handler call counter, and attribute the 5.08 ms warm get variant and 1.16 ms day-cached search gwas gaps before changing behavior. Then change one step at a time, measuring after each: read in deferred transactions with blob re-validation and one retry, keep one connection per process, and skip startup maintenance when nothing is due. Keep SQLite; the performance change introduces no schema change beyond the separately required credential migration.

- Proof: paired 0.9.3-versus-candidate rows from the 2234 method on the same Mac are within 0.5 ms; a stress test with an evictor in one process and readers in another returns only correct bodies or misses; the attribution spans show where the original gap went.

- Changes: `src/cache/manager.rs`, `src/cache/mod.rs`, `src/cache/sqlite.rs`, `src/cache/sqlite/identity.rs`, `src/cache/sqlite/maintenance.rs`, `src/cache/sqlite/pool.rs`, `src/cache/sqlite/read.rs`, `src/cache/sqlite/stress_tests.rs`, `src/cache/sqlite/tests.rs`, `src/cache/timing.rs`, `tests/test_cache_sqlite.py`, `docs/troubleshooting.md`, `CHANGELOG.md`.

## Evidence

- PM decision, 2026-10-21, m3021: build this branch for Ian's risk and milestone decision; do not land it or add it to 2330. Step 5, moving eviction and last_access writes off the request path, stays deferred. The full timing outcome remains unmet until both rows meet the target.
- Historical prototype measurements use 25 alternating 0.9.3/candidate pairs per row on the M5, plus five separate trace samples. All output hashes match 0.9.3 and 2234. These experiments precede the final security composition and file-identity correction; they are not final candidate timings. Median paired differences need not equal the difference between the two sample medians. Values below are milliseconds.

| Step and job | Row | Old median | Prototype median | Median paired difference |
| --- | --- | ---: | ---: | ---: |
| Attribution, j1663 | Warm variant | 11.847583 | 12.857250 | +1.091625 |
| Attribution, j1663 | Day-cached GWAS | 5.423916 | 6.292917 | +0.841125 |
| Deferred read, j1735 | Warm variant | 10.209750 | 11.084583 | +0.803209 |
| Deferred read, j1735 | Day-cached GWAS | 5.750792 | 6.639541 | +0.958126 |
| Shared connection, j1785 | Warm variant | 10.033750 | 10.396459 | +0.301958 |
| Shared connection, j1785 | Day-cached GWAS | 6.398167 | 7.109708 | +0.724375 |
| Startup due-check, j1850 | Warm variant | 9.750250 | 10.069833 | +0.319583 |
| Startup due-check, j1850 | Day-cached GWAS | 6.319625 | 7.082958 | +0.779583 |

- The historical 5.078499 ms variant gap did not reproduce and remains unattributed. Serial trace samples recorded no busy-handler calls; repeated opens/closes were measurable, and connection reuse reduced that cost. Step 4 GWAS trace medians were 523 microseconds opening, 32 reading and 23 startup work. Variant meets the timing target in this prototype; GWAS does not.
- j1833 passed 104 Rust and 16 Python focused checks plus Ruff. j1910 reproduced stale rows after both cache-root and database replacement in an initialized MCP process. Focused repair j1957 passed writer/evictor and forced-unlink retry controls, both replacement cases, the shared-permissions control and Ruff. These are private payload results; no final composed full routine, final paired measurement or Windows identity proof exists yet. Displaced MCP caches must fail closed until restart; ordinary same-identity clearing remains supported.
- Historical steps 1–4 raw evidence is retained on Yellow at `/home/ian/workspace/data/biomcp/ticket-evidence/2326/steps1-4.tar.gz`, 86,681,600 bytes, SHA256 `02f6914b82266e001914349b5ef202a4899b46b8e6cbd822b7afa9418b61a7a0`. The coherent historical source is `a16a77e9d820abe0bd80e3a4dda822379bcb3cf9`; final security composition and risk review remain required.

- sdlc/records/2234-candidate-measurements.md measured median paired increases of 5.078499 ms for warm get variant and 1.164458 ms for day-cached search gwas, with the cause unknown. 0.9.4 accepted them as known costs.
- A 2026-10-21 design review of origin/main 7efd8930d ranked the suspected causes from code reading:
  - Every get runs in an IMMEDIATE transaction (src/cache/sqlite.rs:130-133). Concurrent gets from joined requests then sleep in SQLite's busy handler (busy_timeout 100 ms, sqlite.rs:93) for 1 ms or more.
  - Every get and put opens a fresh connection (src/cache/manager.rs:143, 233), which creates and removes WAL files each time.
  - Startup runs two write transactions even when nothing is due (manager.rs:80-91, sqlite.rs:355-357, 404-405).
- The same review found SQLite's overhead fixable and recommended against switching stores.
- Ian asked on 2026-10-21 whether eviction can run separately, whether faster bulk deletion exists, and for a better long-term design.
