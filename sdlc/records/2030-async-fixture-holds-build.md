# 2030 build record — async fixture holds

Landed 2026-10-08, merge ae9e6c2e9 (9b23236f8 is the follow-up records commit). Review verdicts live on
sdlc/tickets/2030-held-fixture-replies-wait-without-blocking-test-workers.md.

Root cause: held fixture replies blocked an async runtime worker on a
futex (std receive inside tokio::spawn), starving the four-worker
runtime — the 45-minute CI stalls and the intermittent deadline-test
deadlocks. Fix: async mpsc gate under a tokio Mutex; the sibling
thread sleep in the citation fixture became a runtime timer task; the
four hold-capable deadline tests carry a scaled 120-second hang guard
naming the test. Gates: branch CI run 37772471619 green on all jobs;
merged-tree CI green on the landing tree at ae9e6c2e9. An earlier
line here claimed repeated stability runs recorded in the ticket; the
merge dropped that request and no such runs were recorded. The
stability evidence that exists: every main run on a tree with the fix
finished canonical-gates green in 20 to 26 minutes, from the landing
to the pre-tag review. Blocking waits outside the article fixtures
that the review noted stay listed in the deadlock issue for a future
ticket; the interim gate-evidence rule retired before the first full
green main run finished, and that ordering is recorded here honestly.
