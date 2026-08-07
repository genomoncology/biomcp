---
flow: build
priority: 9
---
# Adopt cargo-nextest and pytest-xdist for parallel test execution

Recent biomcp tickets spent 200+ minutes each in quality gates: 173 (SEER) burned 208m, 174 (NIH Reporter) 180m, 175 (WHO) 199m. Ticket 183 hit the 120m code-step cap and timed out because `make spec-pr` alone was eating ~20 minutes per run. The underlying problem: `cargo test` runs serially per crate and `pytest` runs the spec lane without xdist. Switching to `cargo-nextest` and `pytest-xdist -n auto` is a pure infrastructure lever with no coupling to any feature work.

Completed under March on 2026-04-13, as March ticket 187. Imported as history when BioMCP
moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/187-adopt-cargo-nextest-and-pytest-xdist-for-parallel-test-execution
