---
base: b051bade0ca5ed2fee2255ed9d03cc4159964fa7
head: ee61392962de95020af713d6dc113da0658d6a92
---

Added a tracked pre-commit entrypoint and explicit installer. The installed Git
hook contains only a two-line handoff to repository code, so hook behavior is
reviewable and cannot drift into an untracked second implementation.

The entrypoint parses Git's NUL-delimited staged status stream, including both
sides of renames. Root and approved-directory Markdown changes retain the
credential, forbidden-artifact, strict documentation, and relevant spec-lint
checks while skipping Cargo, rustfmt, and Clippy. Empty, mixed, binary, or
unknown staged sets use the complete Rust checks.

All 18 hook tests pass in isolated repositories. They cover each approved
directory, mixed changes, deletes, renames, spaces, a non-UTF-8 path, unknown
extensions, fake Cargo invocation, a credential leak in Markdown, and the thin
installed handoff. Production `src/` did not change.
