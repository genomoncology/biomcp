---
flow: build
priority: 8
---
# Reject non-finite study expression values

Study expression filters accept `NaN` and infinity as thresholds. The command
then succeeds with an apparently meaningful criterion and an empty cohort.
Downloaded expression rows can also admit infinite values. Neither is a
truthful biological measurement boundary.

User thresholds for `--expression-above` and `--expression-below` must be
finite. Non-finite source values must be treated as missing rather than
compared, displayed, or counted. Ordinary finite thresholds and values keep
their existing behavior.

## Done when

- `NaN`, positive or negative infinity, and overflowed numeric forms are rejected as user thresholds.
- Non-finite source rows do not enter study cohorts or output.
- Markdown and JSON return the same structured invalid-argument behavior.
- Finite boundary values continue to filter deterministically.

## Authorized test changes

The design may add or restate assertions in
`src/cli/study/tests/validation.rs`, `src/cli/study/tests/parsing.rs`, the
native tests in `src/sources/cbioportal_study.rs`, and `spec/entity/study.md`.
