---
base: 36ebebca
head: 50194973
---

Made the GenCC generation cleanup keep healthy generations when a
transient environment error hits the classification load, from the
lease-flake issue.

Root cause, found by review rather than by load reproduction: cleanup's
classification loop pushed every `load_generation` failure into the
invalid list, and invalid generations are pruned without the
newest-other retention protection. Under full-suite pressure the load
of a healthy unleased generation fails transiently (file-descriptor or
memory pressure), the generation is deleted, and the lease test's count
drops to two. The child's lease was never implicated; classification
was the only deletion-decision seam. The first design review rejected
the initial fix because the transient errors surface as
`StoreError::Invalid`: `open_existing_at` mapped every failed open to
`Invalid`, and `read_at` mapped read failures to `Invalid`, so variant
discrimination at the classification site alone would have left the
hole open.

The fix has two halves. A pure errno mapping
(`store_error_for_errno`) returns `Unavailable` for environment errnos
(EINTR, EIO, ENOMEM, EMFILE, ENFILE) and `Invalid` otherwise, with a
missing errno reading as `Invalid`; it is applied at exactly the three
transient-capable sites on the load path — the open, its metadata
check's IO failure, and the read — while every deliberate validity
check (mode/uid/nlink, schema, digest, counts) stays `Invalid`. The
classification itself now prunes only `Err(Invalid)`; `Unavailable`,
`Deadline` (routine lease contention), and any other variant retain the
generation for the next publish, with a warning naming the generation
and the error. Pruning an invalid generation also warns, so cleanup
deletions are no longer silent.

Evidence: design REJECT then ACCEPT on revision (the reviewer walked
every error return on the load path and confirmed no transient cause
escapes as `Invalid`); code review ACCEPT with two recorded P2s; the
deterministic regression test
`transient_cleanup_classification_retains_generations` pins three
generations retained while the fault-injected classification fails,
normal retention resuming at two once it clears, and the store loading
throughout; the errno mapping has direct unit tests; the existing
`invalid-finalized` prune test passes unchanged; focused runs of the
lease family green on the gate host; yellow gate at 50194973 — lint,
test, and spec OK. A 120-iteration focused loop and the flake history
stay recorded in the issue file.

Residuals: the non-unix `read_regular` path still maps transient read
failures to `Invalid` (recorded P2; the fix is routing it through the
same taxonomy); the full-suite load reproduction was replaced by the
deterministic seam test plus the reviewer's exhaustive path walk, and
three consecutive full-suite runs at the merged SHA back the fix.
