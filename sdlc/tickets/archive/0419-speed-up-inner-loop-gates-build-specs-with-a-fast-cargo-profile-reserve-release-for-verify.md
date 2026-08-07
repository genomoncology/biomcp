---
flow: quickfix
priority: 10
---
# Speed up inner-loop gates: build specs with a fast cargo profile, reserve --release for verify

Every March build step on biomcp rebuilds the **release** binary to run specs, and `[profile.release]` is tuned for a shipped artifact, not for fast iteration:

Completed under March on 2026-06-15, as March ticket 419. Imported as history when BioMCP
moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/419-speed-up-inner-loop-gates-build-specs-with-a-fast-cargo-profile-reserve-release-for-verify

The landed commit range could not be recovered from git, so no
record accompanies this entry. That is a known gap for the
earliest tickets, not a sign the work is missing.
