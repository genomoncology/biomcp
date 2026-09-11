---
flow: quickfix
priority: 10
---

# Restore the green release baseline

BioMCP keeps the St. Jude talk page and image while the source package is back
at exactly 1,300 files. The README introduction card now follows Quick start,
the full agent index includes the talk page, and two inert Rust tombstones plus
their empty module inventory were removed. Runtime behavior is unchanged.

## Evidence

Independent design and code review accepted the change. The focused package,
agent-index, landing-copy, benchmark-structure, zero-coupling, and quality
ratchet checks passed. `make lint`, `make test`, and `make spec` passed on the
frozen candidate with Node 22; the routine lanes completed 3,375 Rust tests,
954 Python contracts with three skips, strict documentation, and every offline
executable specification group.

## Boundary

This repair does not raise the package ceiling, remove the talk assets, change
runtime behavior, publish a release, or introduce BioData coupling.
