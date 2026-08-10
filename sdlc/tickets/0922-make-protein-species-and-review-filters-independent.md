---
flow: build
priority: 8
deps: ["0915"]
---
# Make protein species and review filters independent

Protein search currently couples two independent filters. Human-only mode also
forces reviewed entries, while `--all-species` silently enables unreviewed
entries. `--reviewed` therefore changes nothing under the default. Users need
to choose species breadth without accidentally changing review quality.

## Filter contract

- Default: human entries and reviewed entries.
- `--all-species`: all species, still reviewed only.
- `--include-unreviewed`: human entries with either review status.
- Both flags: all species with either review status.
- `--reviewed` remains an explicit spelling of the default review scope for
  compatibility and is mutually exclusive with `--include-unreviewed`.

Every compact search row includes a boolean `reviewed` field. Human output
labels unreviewed rows and states the active species and review scope in the
query summary. Help does not imply that one flag changes the other.

## Done when

A four-case local fixture proves the exact UniProt query terms and returned
rows for the matrix above. Clap rejects conflicting review flags before
transport. JSON, Markdown, list output, and public docs describe the same
defaults. No routine test calls UniProt.

## Authorized test changes

Design commits may restate protein CLI and query tests in
`src/cli/protein/tests.rs`, `src/entities/protein.rs`,
`src/sources/uniprot.rs`, protein render tests/templates, and the corresponding
list and documentation contracts. Existing disease, existence, limit, and
offset filters remain independent and covered.

The src line ceiling may rise by at most 150 lines.
