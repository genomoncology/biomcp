---
base: 39f0a0c7
head: e4fc1d5a
---

Raw and typed MCP tools now execute through the CLI's deliberate 8 MiB worker
stack without `RUST_MIN_STACK`. Each request dispatches once. Commands that
need transport metadata carry the JSON projection beside their human output,
so the MCP layer no longer reruns them; article path redaction and provenance
footers still use the matching snapshot.

Inline study charts now fetch and compute once and return their text and SVG
projections together. Errors remain normal MCP tool errors, and the worker is
joined before a call completes.

The stdio and HTTP MCP contract tests passed with the stack environment removed,
including a counting provider, chart calls, full-text redaction, and server
shutdown. Focused chart tests and no-feature Clippy with warnings denied also
passed. The implementation added 147 net `src` lines against the ticket's
220-line ceiling.
