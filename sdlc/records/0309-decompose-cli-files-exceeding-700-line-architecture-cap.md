---
base: c0b9a7b7dc6c89ea30f7f1f23a00c781d835751b
head: b795a469958883f259724ce43dca1557ee7ff2ff
---
Seven CLI module files exceed the 700-line architecture cap documented in `architecture/technical/cli-module-decomposition.md`. The cap exists because oversized CLI files become catch-alls that fight refactors and hide cross-cutting logic. Two prior ticket attempts (180, 181) decomposed `cli/mod.rs` and `render/markdown.rs`; the rest of the surface drifted past the cap since.

Imported from March ticket 309. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/309-decompose-cli-files-exceeding-700-line-architecture-cap
