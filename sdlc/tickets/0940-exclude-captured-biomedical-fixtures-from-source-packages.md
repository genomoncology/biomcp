---
flow: build
priority: 8
deps: ["0937"]
---
# Exclude captured biomedical fixtures from source packages

Issue owned by this ticket:
`sdlc/issues/publishing-this-crate-ships-148-biomedical-fixture-files.md`

`cargo package` currently includes 148 captured provider-response files under
`testdata/`. They are valuable repository tests but are not needed to run the
published binary, and their redistribution status has not been established for
every provider.

This is a precautionary package-boundary decision, not a finding that public
repository hosting is unlawful. The ticket does not settle source-by-source
licensing for the public Git repository. Any repository-removal decision
requires a qualified source/licensing review with provider-specific evidence.

## Package contract

Exclude `testdata/**` from Cargo/crates.io source packages. Keep the fixtures in
the repository and routine test checkout. Do not move them into another
published package or generated archive. A future decision to redistribute a
specific fixture requires a source-by-source licensing/content record and a
separate ticket; silence is not permission.

## Done when

- `cargo package --list` contains zero paths under `testdata/` and none of the
  148 captured response bytes.
- The unpacked package builds the release binary offline from vendored/declared
  source inputs and its runtime help, embedded docs, templates, and skills work.
- Repository `make test` and `make spec` retain their fixture corpus and pass.
- Wheel, native archive, container, and MCPB inspections prove they do not gain
  captured fixtures through another packaging path.
- Source-package and licensing documentation record the safe-default decision.

## Authorized test changes

Design and code commits may add Cargo package include/exclude and artifact-content tests,
adjust source-package documentation, and restate an existing expectation only
where it assumed repository testdata was published. Runtime fixtures and
biomedical behavior must not change.

Delete the named issue when this ticket lands.

The src line ceiling may not rise.
