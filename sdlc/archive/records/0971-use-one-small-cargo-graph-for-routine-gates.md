---
base: a41bbd177ae2f0c2a81191c1c17fe3e3060c53b7
head: 8ab4969c34240a8e91d7e2de206ca6c94a26ff6c
---

Routine Clippy, nextest, and specification preparation now consume the one
declared `--no-default-features` graph. The Makefile exports that choice to the
lint and spec runners, and source contracts reject a routine gate that silently
restores the default AlphaGenome stack.

`make full-feature-check` is the separate shipped-feature proof. It lints all
targets with all features, runs the six AlphaGenome behavior tests, and builds
the all-feature release CLI. The release gate invokes the routine lint/test
path, this named proof, and executable specs against that release binary. CI and
developer documentation describe the two lanes explicitly.

The first full-feature proof exposed and fixed one existing Clippy warning in
the MCP contract example. Its first run populated cold test and release
artifacts in 532.93s; an immediate repeat passed in 1.41s. This expensive proof
is now confined to release readiness.

A fresh external Cargo target measured routine lint at 99.38s and the complete
test gate at 440.36s. The first following spec execution took 197.89s and found
a stale self-documentation assertion after all other spec groups passed. That
probe and two build-identity fixture tests were made independent of outer Cargo
target overrides. On the exact final implementation commit, a sealed warm
lint/test/spec sequence passed in 26.94s, 162.40s, and 186.42s: 375.76s total,
4.27x faster than the 1,606s intervention baseline.
