# 2077 — Upgrade tonic and prost to 0.14

Status: OPEN.

Milestone: 0.9.3

Depends on: 2072

## Outcome

With all features on, the AlphaGenome client builds on tonic and prost 0.14, the tree holds one copy each of tower, indexmap, socket2 and axum, and `deny.toml` has no `rustls-pemfile` advisory exception.

## Evidence

- `cargo tree --all-features` on current main: tower 0.4 and indexmap 1 (with hashbrown 0.12) come only from tonic 0.12; so do socket2 0.5, axum 0.7, matchit 0.7 and `rustls-pemfile`, the `RUSTSEC-2025-0134` exception in `deny.toml`. tonic 0.14.6 depends on tower 0.5, socket2 0.6 and axum 0.8, and has no `rustls-pemfile`.
- Under `--no-default-features` the duplicate check passes trivially, so the proof names its features.
- tonic 0.14 moved prost support into `tonic-prost` and `tonic-prost-build` and renamed its TLS features.
- PM decision 2026-10-14: use `tls-ring` and `tls-native-roots`, which match today's ring and native-roots setup.

- Starts from: `tonic = "0.12"` and `prost = "0.13"` in `Cargo.toml` after 2072 trims tonic's features; `tools/alphagenome-proto-generator/Cargo.toml` (`tonic-build = "=0.12.3"`); `scripts/regenerate-alphagenome-proto`; `src/generated/google.gdm.gdmscience.alphagenome.v1main.rs`; `src/sources/alphagenome.rs`.
- Keeps: AlphaGenome behavior and its fixtures.
- Changes: claims `Cargo.toml` `Cargo.lock` `deny.toml` `tools/alphagenome-proto-generator/**` `scripts/regenerate-alphagenome-proto` `src/generated/**` `src/sources/alphagenome.rs` `tests/test_alphagenome_proto_generation.py` `CHANGELOG.md`.
  - tonic 0.14 with `tls-ring` and `tls-native-roots`, plus `tonic-prost`; prost 0.14; the generator moves to `tonic-prost-build` 0.14; regenerate `src/generated/`.
  - Drop `RUSTSEC-2025-0134` from `deny.toml`.
  - One Internal changelog bullet.
- Proof: `cargo tree -d -e normal --all-features` lists one tower, one indexmap, one socket2 and one axum (each lists two on current main); `cargo deny check` passes without the exception; `scripts/regenerate-alphagenome-proto --check` and `make full-feature-check` pass.
- Defers: nothing.
- Order: after 2072, which edits `Cargo.toml` and `tests/test_alphagenome_proto_generation.py` and removes tonic's server half. It edits `deny.toml` alongside 2078, so the two land one after the other.
