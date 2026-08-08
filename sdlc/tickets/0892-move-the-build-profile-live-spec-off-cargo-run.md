---
flow: quickfix
priority: 3
---
# Move the build-profile live spec off `cargo run`

## Done when

`spec/surface/build-profile-live.md` consumes a prebuilt feature-off
binary supplied by the live runner. No spec invokes Cargo, so the
live-spec lane no longer serialises against the shared `target/` lock.

## The finding

Raised under March and carried over when BioMCP moved to the sdlc
factory. The text below is as filed.

Ticket 643 was authorized to update its assertion but not to change the harness.

Suggested action: as part of ticket 645's profile/spec classification, make the
live-profile spec consume a prebuilt feature-off binary supplied by the live
runner rather than invoking Cargo itself.
