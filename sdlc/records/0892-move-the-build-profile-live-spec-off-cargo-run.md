---
base: 27a535a3d65c85fcdbec34f916897af5aae2b24d
head: 98e4ad23740fa080571a61bad4e6408503f01b60
---

The specification runner now owns one explicit artifact-preparation phase.
Routine modes build the feature-off CLI and MCP example together once, copy
them to stable paths, and capture Cargo dependency-tree and package-metadata
evidence once. Live verification keeps feature-on and feature-off CLIs in
distinct directories and discovers the library test plus six filtered
integration-test executables from one `cargo test --no-run` invocation.

MCP pages, the section-outcome helper, build-profile pages, and filtered Rust
test blocks execute prepared paths directly. The spec-lint audit rejects
build-inducing Cargo commands in executable pages and fixture helpers,
including command substitutions. A runner test proves missing prepared paths
fail clearly without falling through to Cargo or an installed BioMCP binary.
The release path copies its supplied feature-on CLI before the feature-off
build can replace Cargo's profile output.

All 106 focused preparation, quality-ratchet, documentation, and isolation
contracts passed. Full lint passed in 27.98s. A complete four-worker `make
spec` passed in 185.30s with unchanged routine results, compared with the
205.42s post-0968 median and 592.42s intervention baseline. Warm routine
preparation itself took 0.77s.
