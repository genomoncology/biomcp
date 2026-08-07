---
flow: architect
priority: 5
---
# Decompose CLI files exceeding 700-line architecture cap

Seven CLI module files exceed the 700-line architecture cap documented in `architecture/technical/cli-module-decomposition.md`. The cap exists because oversized CLI files become catch-alls that fight refactors and hide cross-cutting logic. Two prior ticket attempts (180, 181) decomposed `cli/mod.rs` and `render/markdown.rs`; the rest of the surface drifted past the cap since.

Completed under March on 2026-04-26, as March ticket 309. Imported as history when this
repo moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/309-decompose-cli-files-exceeding-700-line-architecture-cap
