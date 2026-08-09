---
base: 5086ede3fd960245c4041280b48cdccf41808baf
head: 39390feb6e479d37f6b9b3bb1bb4caaae3fd84a6
---
Unreleased builds reported only the Cargo release version, so binaries from different commits were indistinguishable. The build now appends the short Git SHA off the matching release tag while preserving the plain Cargo version at that tag. CLI, JSON, and Clap version output share the stamped build version, with regression coverage for the public surfaces.

The unrelated long-TMPDIR Unix-socket failure was recorded separately in `sdlc/issues/unix-socket-cache-clear-test-assumes-short-temp-path.md`.
