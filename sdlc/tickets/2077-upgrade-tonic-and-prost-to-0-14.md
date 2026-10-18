---
priority: 3
---

# 2077 — Upgrade tonic and prost to 0.14

Status: OPEN.

Milestone: 0.9.4

Reviews: revision a0b0230, accept

Reviews: revision d545e6e, accept

## Outcome

The shipped AlphaGenome client builds on tonic and prost 0.14, the all-features normal dependency tree has no duplicated tower, indexmap, socket2 or axum versions, and the obsolete rustls-pemfile advisory exception is removed.

Depends on: 2072
Depends on: 2156

- Starts from: tonic/prost in `Cargo.toml`, the existing protocol generator, generated AlphaGenome code and `src/sources/alphagenome.rs`.
- Keeps: client behavior and fixtures. Preserve the approved TLS choices, `tls-ring` and `tls-native-roots`.
- Changes: claims `Cargo.toml` `Cargo.lock` `deny.toml` `tools/alphagenome-proto-generator/**` `scripts/regenerate-alphagenome-proto` `src/generated/google.gdm.gdmscience.alphagenome.v1main.rs` `src/sources/alphagenome.rs` `tests/test_alphagenome_proto_generation.py` `CHANGELOG.md`. Use tonic-prost and tonic-prost-build as required by the approved upgrade; regenerate from the existing protocol inputs.
- Proof: through the build job, inspect `cargo tree -d -e normal --all-features` and the full tree to show each named dependency is present once. `cargo deny check` passes without RUSTSEC-2025-0134; generator `--check` and `make full-feature-check` pass on the exact pushed revision before review, including feature behavior tests. The routine merged-candidate gate passes.
- Defers: nothing.

PM decision: 2026-10-17. Refresh the baseline instead of claiming every named crate still has two versions. Routine no-default-feature checks alone cannot prove this upgrade.

PM decision: 2026-10-17. Moved to 0.9.4. Ian ruled that 0.9.3 ships once every bug is fixed; this ticket improves tests, tooling or dependencies and fixes no user-facing bug. Rebase onto main before resuming; earlier reviews cover the old base only.
