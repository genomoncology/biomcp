# 2078 — Move article HTML to one current HTML stack

Status: OPEN.

Milestone: 0.9.3

Depends on: 2072
Depends on: 2088

## Outcome

Article HTML parsing and readability run on scraper 0.27 and dom_smoothie, so html5ever 0.26, syn 1, the direct `getrandom` and the `fxhash` advisory exception are gone.

## Evidence

- `readability-rust 0.1` has one release and pins `scraper 0.18`, which pulls html5ever 0.26 beside htmd's 0.38: 33 crates, 34.5 s of release CPU, 24.8 s dev (`sdlc/designs/2026-10-13-0-9-3-dependencies-and-dead-code.md`, step 7). On current main `cargo tree -i syn@1` is non-empty, through html5ever 0.26 from both readability-rust and scraper 0.18.
- dom_smoothie 0.18.2 (MIT) uses dom_query 0.28, which uses html5ever 0.39, the same version as scraper 0.27 (ISC, which `deny.toml` allows). `uuid` is already in the tree through rmcp.
- The old proof "at most two html5ever versions" already passes on main (0.26 and 0.38), so the proof names the versions.
- PM decision 2026-10-14: replace readability-rust with dom_smoothie 0.18.

- Starts from: the scraper call sites `src/sources/fda_orphan.rs:10`, `src/transform/article/html.rs:7` and `src/entities/variant/erepo.rs:446`; the Readability call in `extract_readable_html` (`src/transform/article/html.rs:346`); `getrandom::fill` in `unique_suffix` (`src/sources/gencc/store.rs:1067`).
- Keeps: article full-text output, checked against the article fixtures.
- Changes: claims `Cargo.toml` `Cargo.lock` `deny.toml` `src/transform/article/html.rs` `src/sources/fda_orphan.rs` `src/entities/variant/erepo.rs` `src/sources/gencc/store.rs` `CHANGELOG.md`.
  - scraper 0.27; readability-rust replaced with dom_smoothie 0.18.
  - `uuid::Uuid::new_v4()` for the gencc nonce; drop the direct `getrandom`.
  - Drop `RUSTSEC-2025-0057` from `deny.toml`.
  - One Internal changelog bullet.
- Proof: `cargo tree -i syn@1` is empty; `cargo tree -d -e normal --all-features` lists html5ever only at 0.38 and 0.39, with no 0.26 (current main lists 0.26 and 0.38); `cargo deny check` passes without the fxhash exception; the article fixtures and full-text spec pages pass, and any output change is shown in the review.
- Defers: moving htmd to html5ever 0.39 (htmd 0.5.5 still pins 0.38).
- Order: after 2072, which edits `Cargo.toml` and `src/sources/gencc/store.rs`. It edits `deny.toml` alongside 2077, so the two land one after the other. 2088 also edits `src/sources/fda_orphan.rs`; land 2078 after it.
