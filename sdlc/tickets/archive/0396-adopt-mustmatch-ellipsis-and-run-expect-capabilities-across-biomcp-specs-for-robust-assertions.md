---
flow: build
priority: 5
---
# Adopt mustmatch ellipsis and run-expect capabilities across biomcp specs for robust assertions

Once biomcp runs on the mustmatch Rust binary (ticket 393 ports the runner and unpins), the new assertion capabilities become available — most importantly the **line-oriented ellipsis** (`...`), which the pinned 0.0.4 plugin lacks. biomcp's specs assert against multi-line command output (cards, tables, lists) that mixes stable behavioral structure with **volatile content** (result counts, timings, PMIDs, dates, long bodies). Today that forces a brittle choice: either pin exact volatile values (breaks on drift) or assert only fragments. Ellipsis resolves it: anchor on the lines that encode behavior, elide the rest. This ticket upgrades the spec corpus to use ellipsis (and the run/expect block separation where it helps), making biomcp's specs robust and an exemplar of the new mustmatch approach — eliminating brittleness as a flake source alongside the live-API determinism work (395).

Completed under March on 2026-06-05, as March ticket 396. Imported as history when BioMCP
moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/396-adopt-mustmatch-ellipsis-and-run-expect-capabilities-across-biomcp-specs-for-robust-assertions
