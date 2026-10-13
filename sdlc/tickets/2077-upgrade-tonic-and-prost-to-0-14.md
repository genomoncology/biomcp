# 2077 — Upgrade tonic and prost to 0.14

Status: OPEN.

Milestone: 0.9.3

## Outcome

The AlphaGenome client builds on current tonic and prost, with one copy each of tower, indexmap and socket2, and no `rustls-pemfile` advisory exception.

## Evidence

- tonic 0.12 keeps tower 0.4, indexmap 1, hashbrown 0.12 and socket2 0.5 beside their current versions, and the `RUSTSEC-2025-0134` exception in `deny.toml` (`sdlc/designs/2026-10-13-0-9-3-dependencies-and-dead-code.md`, step 6).

- Starts from: 2072 (tonic features trimmed).
- Keeps: AlphaGenome behavior and its fixtures.
- Changes: tonic and prost 0.14; regenerate `src/generated/` with the repository's proto generator; drop the advisory exception.
- Proof: `cargo tree -d` shows one tower and one indexmap; `cargo deny check` passes without the exception; the generated-sources CI job passes; AlphaGenome fixture tests pass.
- Defers: nothing.
