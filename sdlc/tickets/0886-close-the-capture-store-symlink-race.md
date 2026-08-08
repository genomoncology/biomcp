---
flow: build
priority: 7
---
# Close the capture store's symlink race

## Done when

A symlink swapped into a staging or derived shard directory between the
check and the write cannot cause a write outside the capture store. A
test exercises the swap at the moment the current code is vulnerable and
fails against today's implementation.

## Why here, why now

Priority 7 for being the only security-shaped item in the backlog: it
writes outside its intended tree, and the current mitigation is a check
that a race can step around.

## The finding

Raised under March and carried over when BioMCP moved to the sdlc
factory. The text below is as filed.

## Detail
`ProviderCaptureStore` validates derived directory components with `symlink_metadata`, then creates or writes using path-based standard-library calls. A process that can modify the managed cache root between those operations could replace a component with a symlink and redirect a staging/blob/metadata write outside the capture namespace. The ticket's direct pre-existing-symlink case is covered; closing the race needs descriptor-relative no-follow operations rather than another path check.

## Suggested action
Implement directory-descriptor-relative capture publication (for example `openat` with no-follow semantics on supported platforms), preserving the current derived-layout checks. Add a Unix race/harness or deterministic descriptor-level regression test if one can prove the protection without timing flakiness. Intended improved-test destination: experiment/harness plus native test.
