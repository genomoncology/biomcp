---
base: 76e487c1
head: f645958a
---

Both study expression filters now reject every parsed non-finite value and
exponent overflow before study lookup while preserving the finite Rust `f64`
grammar and boundaries. Non-finite downloaded expression cells are treated as
missing in distributions, sample maps, cohorts, comparisons, and paired data,
without dropping neighboring finite cells.

CLI JSON, raw MCP, source parsing, and fixture-backed study specifications
passed. Independent review accepted the implementation and its unchanged
source-size baseline.
