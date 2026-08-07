---
base: d17fc4658177a1c934d6eb4135e140d136231433
head: 3530a7b15fef7fbc37ca3bb57a67d4f29d50b692
---
Recent biomcp tickets spent 200+ minutes each in quality gates: 173 (SEER) burned 208m, 174 (NIH Reporter) 180m, 175 (WHO) 199m. Ticket 183 hit the 120m code-step cap and timed out because `make spec-pr` alone was eating ~20 minutes per run. The underlying problem: `cargo test` runs serially per crate and `pytest` runs the spec lane without xdist. Switching to `cargo-nextest` and `pytest-xdist -n auto` is a pure infrastructure lever with no coupling to any feature work.

Imported from March ticket 187. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/187-adopt-cargo-nextest-and-pytest-xdist-for-parallel-test-execution
