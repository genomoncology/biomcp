---
flow: build
priority: 3
deps: []
---

# 1218: Study errors name the missing file

## Goal

A failed `study` command says which file is missing. Today every failure prints `Source unavailable: cBioPortal DataHub is not available.` in Markdown and in JSON, and a user cannot tell a missing `data_mutations.txt` from a missing `meta_study.txt` from an uninstalled study. The reason already exists inside the error and is thrown away at the renderer.

```
biomcp study compare --study gse48843_counts --gene TP53
Error: Source unavailable: cBioPortal DataHub is not available (data_mutations.txt).
```

The change is global to the `study` family. It is its own ticket because it touches every existing study command, not the GEO import that found it. Tickets 1209 and 1210 depend on it, so 1210 can land without waiting for the whole import.

## Current Facts

- `BioMcpError::SourceUnavailable` carries `source_name`, `reason`, and `suggestion` (`src/error.rs:404`). The Markdown renderer prints `Source unavailable: {source} is not available.` and drops `reason` (`src/error.rs:482`). The JSON renderer prints the same sentence plus `Check source setup and retry.` (`src/error.rs:573`).
- The study source builds that error at fifteen sites in `src/sources/cbioportal_study.rs` (`:299`, `:489`, `:538`, `:719`, `:814`, `:926`, `:1058`, `:1139`, `:1189`, `:1241`, `:1484`, `:2117`, `:2156`, `:2170`, `:2192`). `parse_meta_study` (`:1483`) builds one with the reason `Missing study metadata file: <path>` (`:1484`).
- The reason holds an absolute local path today, through `path.display()` (`src/sources/cbioportal_study.rs:1486`). MCP withholds workstation-local paths, so the reason cannot be rendered as it stands.
- The same renderer serves every source. `spec/entity/section-outcomes.md:175` and `:177` pin the exact DDInter strings, in Markdown and in JSON, and `spec/entity/diagnostic.md:116` pins the MyDisease.info one. A change to the shared sentence would break all three.
- `spec/surface/cli-contract-ratchet.md` runs under `make spec` (`scripts/run-specs.sh:42`) and routes the whole-surface ratchet through `tests/test_cli_surface_contract_ratchet.py` under `make test`. The error line is not in the ratchet's registry today, and the implementer confirms the ratchet accepts a changed error line before landing.

## Design

- `SourceUnavailable` gains one optional field, `detail: Option<String>`. Every existing construction leaves it `None` and renders byte for byte as it does today, so DDInter, MyDisease.info, and every other source keep their pinned strings.
- The Markdown renderer prints `Source unavailable: {source} is not available ({detail}).` when `detail` is set, and the current sentence when it is not (`src/error.rs:482`). The JSON renderer does the same in its `message` (`src/error.rs:573`), and the recovery sentence is unchanged.
- `detail` is a bare file name, never a path: `meta_study.txt`, `data_mutations.txt`, `data_clinical_sample.txt`, or the expression file name. The absolute path stays out of it, because the same message travels over MCP. The existing `reason` keeps the full path for logs and is still not rendered.
- The study source sets `detail` at each site that is about one file, with the file name it was looking for. A site that is not about one file, such as the study root not being a directory (`src/sources/cbioportal_study.rs:299`), leaves `detail` as `None` and renders as it does today.
- No command computes anything new, no exit code changes, and no error class changes. `ExternalFailureProjection` (`src/error.rs:445`) is untouched.

### Docs

- `docs/user-guide/study.md` and the troubleshooting page show the new form and say which file each message names.
- `CHANGELOG.md` gains an entry.

## Acceptance

Fixture-backed Rust tests, no live network:

1. A study folder with no `meta_study.txt` fails with a Markdown message naming `meta_study.txt` and a JSON `message` naming the same, and neither carries an absolute path. A test pins both strings.
2. `study compare --gene` on a study with no `data_mutations.txt` names `data_mutations.txt` in both forms.
3. A `SourceUnavailable` with `detail: None` renders byte for byte as it does today, in Markdown and in JSON. A test pins the DDInter and MyDisease.info strings that the specs already pin.
4. Every existing study test passes unchanged except the ones that assert the bare sentence for a study failure, and each of those is updated to the new string in the same commit.
5. `spec/entity/section-outcomes.md` and `spec/entity/diagnostic.md` pass unchanged.
6. The whole-surface ratchet accepts the changed error line, or its registry is updated in the same commit and the change is named there.
7. The JSON error object keeps its `code`, `source`, and `recovery` fields, and only `message` differs.

`make lint`, `make test`, and `make spec` pass on the gate host at the pushed SHA.

## Out of scope

- Rendering `reason` for any other source. Every other construction keeps `detail: None`.
- Exposing the absolute path in any rendered message.
- New error classes, new exit codes, and any change to what a command computes.
- The missing-file behavior itself. A missing file still fails; this ticket changes only what the failure says.

## Decisions

Open to Ian's overturn.

1. The detail rides in a new optional field rather than by rendering the existing `reason`. `reason` carries an absolute path and is written for logs, and three executable specs pin the current sentence for other sources.
2. The detail is a bare file name. The same message reaches MCP, which withholds workstation-local paths.
3. This is its own ticket rather than part of ticket 1209, because it changes every study command's failure output and ticket 1210 needs it without needing the import.

## Review

- Design review: pending
- Code review: pending
