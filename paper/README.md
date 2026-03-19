# Paper Package

This directory holds the repo-facing paper package for BioMCP. It documents the
reviewer-visible file layout, the placeholder archive schemas committed now, and
the reproduction scripts operators can run from this repository.

## Layout

```text
paper/
  README.md
  data/
  supplementary/
  scripts/
```

- `paper/data/` currently contains placeholder schemas for archived release
  payloads. These files are committed stubs now and are replaced with the final
  release pack before publication.
- `paper/supplementary/` contains reviewer-facing stub tables that describe the
  expected columns for the paper appendix.
- `paper/scripts/` contains reproduction helpers for the traceability audit,
  workflow reruns, normalization checks, and token measurement.

## File families

### Supplementary tables

- `table-s1-sources.md`: leaf-style source enumeration
- `table-s2-comparison.md`: landscape comparison matrix
- `table-s3-stress-test.md`: normalization and notation-acceptance stress test
- `table-s4-source-citations.md`: source citation table
- `table-s5-token-cost.md`: compact versus naive token-cost measurements
- `table-s6-engineering.md`: engineering and health metrics

### Data files

- `traceability-audit.json`: stub schema for the archived claim verification audit
- `workflow-adjudication.json`: stub schema for archived workflow prompts, scores, and runtimes
- `normalization-benchmark.json`: stub schema for alias, brand-name, and notation checks
- `token-cost.json`: stub schema for archived workflow token and byte totals
- `conflict-cases.json`: stub schema for overlapping-source review cases
- `health-snapshot.json`: stub schema for the archived service health capture

### Scripts

- `run-traceability-audit.sh` is runnable immediately against the current repo
  and writes captures under `paper/generated/traceability` by default.
- `run-workflows.sh` and `run-normalization.sh` require archived release data in
  `paper/data/` before they can run. They refuse to execute against stub files.
- `measure-tokens.py` reads generated workflow outputs and summarizes token and
  byte counts.

## Output location

Reproduction outputs are written to `paper/generated/` or to an explicit output
directory you pass as the first positional argument. Normal script execution
does not write back into `paper/data/`.

## Binary discovery

The shell scripts resolve the BioMCP binary in this order:

1. `BIOMCP_BIN`, if it points to an executable file
2. `./target/release/biomcp`, if that release binary exists
3. `biomcp` on `PATH`

If no binary is found, the scripts exit with a clear error.

## Current workflow

Use the scripts from the repository root or from inside `paper/scripts/`.

```bash
bash paper/scripts/run-traceability-audit.sh
bash paper/scripts/run-workflows.sh
bash paper/scripts/run-normalization.sh
./paper/scripts/measure-tokens.py
```

Today, only `run-traceability-audit.sh` is runnable immediately. The other two
shell scripts are wired for the archived release pack and will tell the
operator to replace the stubs in `paper/data/` before continuing.
