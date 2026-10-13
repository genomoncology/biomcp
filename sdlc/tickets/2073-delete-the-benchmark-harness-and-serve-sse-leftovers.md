# 2073 — Delete the benchmark harness and serve-sse leftovers

Status: OPEN.

Milestone: 0.9.3

## Outcome

Code that never ships and commands that no longer work are gone.

## Evidence

- `src/cli/benchmark/` (2,560 lines) compiles only under `#[cfg(test)]`, has no command, and nothing outside it refers to it (`sdlc/designs/2026-10-13-0-9-3-dependencies-and-dead-code.md`, Dead code).
- `serve-sse` is a hidden command that only prints a removal message; `main.rs` keeps two match arms for it and four Python tests assert the message.
- One tuple field in `src/sources/reactome.rs` has no reference anywhere.

- Starts from: `sdlc/designs/2026-10-13-0-9-3-dependencies-and-dead-code.md`, step 5, first half.
- Keeps: the test-only items that construction tests still use; they go with 2081.
- Changes: delete the benchmark module and its decision note's pointer; delete `serve-sse` and its tests, so the CLI reports it as an unknown command; delete the reactome field; remove their entries from `tools/dead-code-exceptions.json`.
- Proof: the build and every lane pass; `biomcp serve-sse` exits nonzero as an unknown command; the dead-code exception list shrinks by those entries.
- Defers: the 63 test-only items and 58 unread fields (2081).
