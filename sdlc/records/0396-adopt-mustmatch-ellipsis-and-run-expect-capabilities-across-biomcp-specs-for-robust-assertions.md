---
base: e8c7f0f04e4eeb2b531e67d1e0741b11f7a645ee
head: c4f91ed6fa93d354499c8aab184ca70aa7785d27
---
Once biomcp runs on the mustmatch Rust binary (ticket 393 ports the runner and unpins), the new assertion capabilities become available — most importantly the **line-oriented ellipsis** (`...`), which the pinned 0.0.4 plugin lacks. biomcp's specs assert against multi-line command output (cards, tables, lists) that mixes stable behavioral structure with **volatile content** (result counts, timings, PMIDs, dates, long bodies). Today that forces a brittle choice: either pin exact volatile values (breaks on drift) or assert only fragments. Ellipsis resolves it: anchor on the lines that encode behavior, elide the rest. This ticket upgrades the spec corpus to use ellipsis (and the run/expect block separation where it helps), making biomcp's specs robust and an exemplar of the new mustmatch approach — eliminating brittleness as a flake source alongside the live-API determinism work (395).

Imported from March ticket 396. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/396-adopt-mustmatch-ellipsis-and-run-expect-capabilities-across-biomcp-specs-for-robust-assertions
