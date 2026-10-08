# 2030 build record — async fixture holds

Landed 2026-10-08, merge 9b23236f8. Review verdicts live on
sdlc/tickets/2030-held-fixture-replies-wait-without-blocking-test-workers.md.

Root cause: held fixture replies blocked an async runtime worker on a
futex (std receive inside tokio::spawn), starving the four-worker
runtime — the 45-minute CI stalls and the intermittent deadline-test
deadlocks. Fix: async mpsc gate under a tokio Mutex; the sibling
thread sleep in the citation fixture became a runtime timer task; the
four hold-capable deadline tests carry a scaled 120-second hang guard
naming the test. Gates: branch CI run 37772471619 green on all jobs;
merged-tree CI green at 9b23236f8; the builder's repeated-scope
stability runs green via the build host (recorded in the ticket).
