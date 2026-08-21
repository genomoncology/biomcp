---
flow: build
priority: 12
---
# The suite builds the crate once, inside the tree

The packaging test (the `test_verified_package_compiles` family)
extracts the packaged crate into pytest's temp area and compiles it
from scratch — a second complete debug build of the same crate,
~4G with a ~600MB rlib, on every run, beside the working tree's own
build. On 2026-08-21 that landed in a stage's temp directory and
helped fill the machine's disk. The double build is waste on any
machine: same crate, same profile, built twice into two places and
both kept.

Done, observably: one full run of the test suite compiles the crate
once — the packaging test proves what it means to prove (the
packaged crate as shipped actually compiles) while sharing the
build directory with the tree's own build, so the second multi-
gigabyte target tree no longer exists. And every artifact the suite
creates — pytest's temp area included — lives under the working
tree, not in the environment's temp directory, so it dies when the
tree is torn down. The suite is measurably faster; the packaging
test's guarantee is not weakened, and if full isolation of some
step is genuinely load-bearing, the design says which step and why
it earns a separate build.

The behavior replaced: the packaging test's from-scratch build in
pytest's default temp location, and any assertion pinning those
paths or that isolation, restated to the shared, in-tree layout.
