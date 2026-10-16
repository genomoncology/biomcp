---
flow: build
priority: 0
deps: []
---

# 2143: Upgrade zip past the symlink advisory

Status: OPEN.

Milestone: 0.9.3

Reviews: revision 3986d90, accept

## Outcome

`cargo deny check advisories` passes again, so `make lint` and ci.yml are green on main.

## Evidence

- 2026-10-16: ci.yml lint fails on every push. `cargo deny check advisories` flags zip 0.6.6 (`Cargo.lock:437`): `ZipStreamReader::extract()` applies Unix permissions through extracted symlinks without checking the target stays inside the destination (GHSA-4684-h2q9-2vcp). The fix is zip 9.0.0 or later.
- `Cargo.toml:109` pins `zip = "0.6"` with only `deflate`. BioMCP reads archives in `src/cli/update.rs` (self-update) and `src/sources/europepmc.rs`, and writes test archives in their tests. Neither calls `extract()`, so BioMCP is not exposed, but the lint gate fails regardless.

## Work

1. Move zip to the current release (9.0.0 or later), keeping `default-features = false` and deflate only.
2. Adapt the four call sites to the new API. Change no behavior.
3. Do not add an advisory ignore.

## Acceptance

1. `make lint` passes, including `cargo deny check advisories`.
2. The self-update and Europe PMC archive tests pass unchanged in what they assert.
3. ci.yml is green on the landing tip.
4. The changelog carries a bullet for this ticket.
