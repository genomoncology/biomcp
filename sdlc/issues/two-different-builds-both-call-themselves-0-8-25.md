# Two different builds both report version 0.8.25

Severity: should-fix. It caused two wrong issue reports in one
sitting.

The binary on PATH and the binary built from this repo report the
same version and expose different commands.

    $ biomcp --version                     # ~/.local/bin/biomcp
    biomcp 0.8.25
    $ ./target/release/biomcp --version
    biomcp 0.8.25

    $ biomcp gene --help | grep cspec      # nothing
    $ ./target/release/biomcp gene --help | grep cspec
      cspec  Retrieve versioned ClinGen Criteria Specification Registry source documents

The installed one predates the `gene cspec` work. `Cargo.toml` still
reads `version = "0.8.25"`, so every build between releases inherits
the last published number.

## What it cost

Researching PTEN GN003, `gene cspec` and `variant erepo` looked
absent. Both were filed as feature requests. Both exist and are
well-built — versioned captures with content hashes, criteria
parsed, evidence codes at applied strength. Two issues had to be
rewritten down to the narrow gaps that are actually real
(`feature-clingen-criteria-specifications-as-an-entity.md`,
`feature-clingen-expert-panel-assertions.md`).

Checking `biomcp list` and `--help` first is the documented
discipline and it was followed. It gave the wrong answer, because
the tool answering was not the tool in the repo.

## Fix shape

- Put the commit in the version string for non-release builds —
  `0.8.25+g28cd0036` or `0.8.26-dev.g28cd0036`. `build.rs` already
  exists, so the git describe is cheap. This alone closes it.
- Consider bumping `Cargo.toml` to the next patch with a `-dev`
  suffix immediately after each release, so an unreleased build
  never claims a released number.
- `biomcp --version` could also print the binary's own path. A
  stale copy earlier on `PATH` than the intended one is the usual
  cause and is invisible today.

Not a research-only problem: any bug report against "0.8.25" is
ambiguous about which 0.8.25, including reports from outside.

Found 2026-08-08.
