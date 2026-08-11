---
flow: build
priority: 10
deps: ["0951"]
---
# Use one small Cargo graph for routine gates

Routine tests and specs use `--no-default-features`, but `make lint` enables the
default AlphaGenome gRPC/protobuf stack. Ordinary validation compiles two
different dependency graphs and normal lint runs a generator for a feature the
routine corpus cannot exercise.

## Test contract

Make the routine feature selection explicit in one place and prove lint, test,
and spec consume it. Keep a separate named full-feature gate and prove release
validation invokes it.

## Done when

- Routine Clippy, nextest, and spec preparation use the same declared feature
  graph and do not include tonic/prost/zstd/tonic-build through AlphaGenome.
- One release/full-feature check compiles and lints every shipped feature and
  retains the AlphaGenome behavior tests.
- A source contract fails if an ordinary gate silently restores defaults or a
  release gate silently selects the smaller graph.
- Dependency and target reuse is measured for cold setup and a warm sequential
  lint/test/spec run.
- Developer and CI documentation name the two lanes without implying routine
  coverage exercises AlphaGenome.

## Authorized test changes

Design commits may restate Makefile feature variables, build-profile specs, CI
commands, and feature-selection tests. No shipped feature is removed.

The src line ceiling may not rise.
