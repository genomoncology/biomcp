# Close provider-capture directory symlink TOCTOU race

Severity: should-fix.

Carried over from March, where it was raised against ticket 616
on 2026-07-23 and left open. The text
below is as filed.
## Summary
The capture store now rejects pre-existing symlinked staging and derived shard directories before publishing a capture, but its check-then-create/write sequence can still be raced by a local filesystem attacker.

## Detail
`ProviderCaptureStore` validates derived directory components with `symlink_metadata`, then creates or writes using path-based standard-library calls. A process that can modify the managed cache root between those operations could replace a component with a symlink and redirect a staging/blob/metadata write outside the capture namespace. The ticket's direct pre-existing-symlink case is covered; closing the race needs descriptor-relative no-follow operations rather than another path check.

## Suggested action
Implement directory-descriptor-relative capture publication (for example `openat` with no-follow semantics on supported platforms), preserving the current derived-layout checks. Add a Unix race/harness or deterministic descriptor-level regression test if one can prove the protection without timing flakiness. Intended improved-test destination: experiment/harness plus native test.
