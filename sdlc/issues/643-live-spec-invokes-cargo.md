# Move the build-profile live spec off cargo run

Severity: nice-to-have.

Carried over from March, where it was raised against ticket 643
on 2026-08-03 and left open. The text
below is as filed.
`spec/surface/build-profile-live.md` invokes `cargo run` inside its mustmatch
block. That duplicates build responsibility and serializes the live-spec lane
against Cargo's shared `target/` lock.

Ticket 643 was authorized to update its assertion but not to change the harness.

Suggested action: as part of ticket 645's profile/spec classification, make the
live-profile spec consume a prebuilt feature-off binary supplied by the live
runner rather than invoking Cargo itself.
