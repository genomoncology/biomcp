---
base: 36ebebca
head: 14906dce
---

Made the GenCC lease test deterministic and closed a latent cleanup
hole it exposed, from the lease-flake issue.

The observed flake, found by combining the new cleanup tracing with the
gate logs: the lease interlock works. The child helper that holds g1's
lease waits for the release file behind a five-second deadline; under
full-suite load the parent's publishes stall past that window, the
child's assert fires, the child exits, and the lease legitimately
releases. The next publish's cleanup then correctly prunes the now
unleased g1, the count drops to two, and the test fails with no warning
lines — exactly the signature of every occurrence (four gate failures,
zero reproduction in 120 focused iterations, which carry no load). The
fix is test policy: the child's hold deadline and the parent's
entered-wait deadline move to 120 seconds with a comment naming the
load-stall reason, and the parent asserts the child is still running
immediately before the deferred-cleanup count, so a future premature
exit reports itself instead of surfacing as a count mismatch.

The first half of the ticket closed a real hole the same investigation
proved exists but that was not this flake: cleanup's classification
loop pushed every `load_generation` failure into the invalid list, and
invalid generations are pruned without the newest-other retention
protection, so a transient environment error during classification
deletes a healthy unleased generation. The first design review rejected
the naive fix because the transient errors surface as
`StoreError::Invalid` — `open_existing_at` mapped every failed open to
`Invalid`, and `read_at` mapped read failures to `Invalid`. The fix
adds a pure errno mapping (`store_error_for_errno`: EINTR, EIO, ENOMEM,
EMFILE, ENFILE read as `Unavailable`; anything else, including a
missing errno, as `Invalid`) applied at the three transient-capable
load-path sites, with every deliberate validity check unchanged; the
classification then prunes only `Err(Invalid)` and retains every other
variant with a warning naming the generation and error. Pruning an
invalid generation also warns, so cleanup deletions are no longer
silent.

Evidence: design REJECT then ACCEPT (the reviewer walked every error
return on the load path and confirmed no transient cause escapes as
`Invalid`); code review ACCEPT with two recorded P2s; the deterministic
regression test `transient_cleanup_classification_retains_generations`
pins three generations retained while the fault-injected classification
fails and normal retention resuming once it clears; direct errno-mapping
unit tests; the existing `invalid-finalized` prune test passes
unchanged; focused lease-family runs green on the gate host; yellow
gate — see the merge record for the final SHA. The first post-fix gate
run reproduced the flake once with zero warning lines, which is the
evidence that separated the deadline cause from the classification
cause.

Residuals: the non-unix `read_regular` path still maps transient read
failures to `Invalid` (recorded P2; the fix is routing it through the
same taxonomy); three consecutive green full-suite runs at the merged
SHA back the deadline fix.
