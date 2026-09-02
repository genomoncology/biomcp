---
---
# Replace the yanked chacha20 dependency

`make lint` reports that `Cargo.lock` resolves `chacha20` 0.10.0, which has
been yanked. The dependency arrives through `rand` 0.10.1 and `rmcp` 1.7.0.
Update the dependency graph to a supported release while preserving the locked,
offline routine gates.
