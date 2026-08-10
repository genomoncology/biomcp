---
flow: build
priority: 2
deps: ["0934", "0957"]
---
# Route doc-only pre-commit checks through a tracked hook

The installed .git/hooks/pre-commit file is local state and does not run the
credential scan claimed by the old ticket. Make the repository own the hook
behavior before optimizing it.

## Done when

A tracked pre-commit entrypoint classifies the staged paths. When every staged
file is Markdown under sdlc, docs, architecture, spec, skills, or the
repository root:

- credential and forbidden-artifact scans still run;
- Markdown/spec static checks relevant to those files still run;
- Cargo, rustfmt, and Clippy do not run.

If any staged file falls outside that exact set, the existing full Rust
pre-commit checks run. Renames, deletions, spaces, and non-UTF-8 path quoting
are handled from git's staged-file output rather than shell word splitting.

## Delivery

Add a tracked installer that puts the entrypoint into .git/hooks/pre-commit
for this checkout and documents the command in CONTRIBUTING.md. Tests use an
isolated temporary repository; they never modify a developer's real hook.

## Proof required

- table-driven staged-path tests cover every allowed directory, mixed
  doc/code changes, deletes, renames, and an unknown extension;
- a fake Cargo executable proves doc-only cases never invoke it and mixed
  cases do;
- a fake leaked credential in Markdown proves scans still run;
- the installed hook is a thin handoff to the tracked entrypoint, not a
  second implementation.

The src line ceiling may not rise.
