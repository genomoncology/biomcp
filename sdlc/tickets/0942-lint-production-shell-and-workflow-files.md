---
flow: build
priority: 6
deps: ["0892", "0934", "0936"]
---
# Lint production shell and workflow files

The canonical lint gate checks Rust and Python thoroughly but has no complete
syntax/static-analysis boundary for installers, release scripts, supervisor
helpers, or GitHub workflows. Current `bash -n` coverage reaches only a small
example subset.

## Gate contract

Add to canonical `make lint`:

- `bash -n` over every tracked file identified as Bash by its shebang or
  explicit repository inventory;
- a pinned ShellCheck over production installer, release, fixture lifecycle,
  supervisor, and gate scripts, with narrow documented per-line exceptions;
  and
- a pinned `actionlint` over every tracked GitHub Actions workflow.

Generated/vendor files are excluded only by one checked inventory with a
written reason. Required tools are installed by hosted CI from pinned versions;
missing tools fail rather than skip. The checks do not execute scripts, contact
providers, mutate the queue, or expand into a generic style rewrite.

## Done when

- Positive tests enumerate every tracked shell/workflow file into exactly one
  checked or reasoned-excluded class.
- Negative fixtures prove a Bash syntax error, a high-confidence ShellCheck
  safety error, and an invalid workflow each fail `make lint`.
- Existing intentional shell portability choices carry the smallest local
  suppressions with explanations.
- CI from ticket 0934 invokes this through the canonical target without a
  duplicated YAML-only check.

## Authorized test changes

Design commits may restate `bin/lint`, Makefile/tool bootstrap, shell/workflow
contract tests, and CI tool installation. Fixes to existing scripts are limited
to findings from the new pinned checks and must preserve their behavioral
contracts. No product Rust source change belongs here.

The src line ceiling may not rise.
