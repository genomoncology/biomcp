---
flow: quickfix
priority: 8
deps: []
---

# Silence the release-only GenCC crash-injection warning

## Goal

`cargo build --release` prints zero warnings while the debug-only
crash-injection seam keeps its exact behavior.

## Current behavior

On current main, `cargo build --release` prints one `unused_variables` warning
for the `error` parameter of `injected` at `src/sources/gencc/store.rs:951`.
The injection body is `#[cfg(debug_assertions)]`, so the release body is empty
and `error` is unused. The file already discards `point` with `let _ = point;`
at `store.rs:965`. Debug builds and `make lint` are unaffected.

## Required behavior

Replace the discard at `src/sources/gencc/store.rs:965` with
`let _ = (point, error);`, matching the repository's existing discard style.

## Observable success

A clean release build prints zero warnings. Debug crash-injection behavior and
all GenCC tests are unchanged. Standard gates pass.

## Boundaries

No behavior change. No public API change. One line of production code.
