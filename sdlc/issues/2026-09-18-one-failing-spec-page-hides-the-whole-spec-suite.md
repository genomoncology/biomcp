# One failing spec page hides the whole spec suite

Filed 2026-09-18 after `make spec` reported a pass count that could not change.

## Symptom

`make spec` reports `174 passed, 1 failed, 8 skipped` and exits 2. The one
failure is the `Release Prep Pins The Development Candidate Version` block in
`spec/surface/mcp.md` (line 994). That summary is not the suite. It is the
article group alone. No entity spec page runs at all, and neither do the Python
contracts.

The suite has been inert for as long as that block has been failing. A spec page
can be wrong, or missing from a registry, and `make spec` still prints the same
numbers.

## Reproduction

On the gate host at `69cc7c31`, with a clean tree:

```
make spec                                   # 174 passed, 1 failed, 8 skipped
sed -i "s/CVCL_0064/CVCL_9999/" spec/entity/cell-line.md
make spec                                   # 174 passed, 1 failed, 8 skipped
```

A deliberately wrong accession in a routed spec page changes nothing. The full
log never mentions that page, not even a pass summary, although
`run_markdown_specs` prints every page log unconditionally.

Removing the failing page from the routine list unmasks the rest:

```
sed -i "s|^  spec/surface/mcp.md$|  # spec/surface/mcp.md|" scripts/run-specs.sh
make spec
```

The run goes from about 110 seconds to over ten minutes. 29 markdown spec pages
execute and every one passes. One Python contract then fails,
`test_ticket_673_runner_is_the_only_complete_spec_registry`, because a spec page
was registered in `scripts/run-specs.sh` but not in the mirrored
`ROUTINE_SPEC_PATHS` constant in
`tests/surface/test_parallel_isolation_contract.py`. That registry gap had been
invisible.

## Cause

`scripts/run-specs.sh` runs `run_article_markdown_specs` before
`run_markdown_specs`, `run_section_outcome_specs`, and `run_python_contracts`.
`spec/entity/article.md`, `spec/entity/author.md`, and `spec/surface/mcp.md`
share one article server and its mutable request log, so they run serially in a
single Mustmatch invocation. When that invocation exits non-zero the script
stops under `set -e`, and the later stages never start.

The article group runs first and fails closed for the whole suite. Any failure
in those three pages hides every other page.

## Why the article page fails

`scripts/check-version-sync.sh` exits 1 with `breaking changes require a minor
version increase before 1.0`. `CHANGELOG.md` lists breaking changes while the
published tag is `v0.9.0`, so the check wants 0.10.0. Reproduced at `51811699`,
before any cell-line work, so this is not new.

That is a separate problem from the masking. Fixing the version pin would clear
today's symptom and leave the masking in place for the next failure.

## Suggested fix

Two changes, independent of each other.

1. Do not let one stage abort the run. Collect the exit status of each stage,
   run every stage, and fail at the end with each failing page named. The
   parallel batch loop already returns 1 on a batch failure, so the same
   collect-and-continue rule belongs there.
2. Print the page name beside each summary. The current output is a list of
   bare counts with no page names, which is why a missing page reads exactly
   like a passing one.

A cheaper interim guard: assert a floor on the total block count, so a run that
executes a fraction of the suite fails rather than reporting a green-looking
number.

## Impact

Any spec-page regression since the changelog problem appeared would not have
been caught. The Rust tests are unaffected and did run.

## Resolved

`scripts/run-specs.sh` now runs every stage and fails at the end.

- `run_spec_stage` wraps each of `run_article_markdown_specs`,
  `run_markdown_specs`, `run_section_outcome_specs`, and
  `run_python_contracts`. A stage that fails is recorded in
  `FAILED_SPEC_ENTRIES` and the next stage still starts.
- The parallel page loop no longer returns on the first failing batch. It
  records each failing page by path and keeps launching the remaining batches.
- Every page log is preceded by `=== spec page: <path> ===`, and the serial
  stages print `=== spec pages: <paths> ===`. A missing page no longer reads
  like a passing one.
- The run ends with a named list of every failure and exits 1.

Observed with `spec-contracts` (article group plus `spec/surface/skills.md` and
`spec/surface/trial-retirement.md`):

- Before: `173 passed, 2 failed, 8 skipped`, exit 1. The two remaining pages
  never ran.
- After: the same article result, then `spec/surface/skills.md` 6 passed and
  `spec/surface/trial-retirement.md` 10 passed. 189 blocks executed instead of
  183.
- A deliberate break in `spec/surface/skills.md` reported
  `page spec/surface/skills.md (exit 1)` in the final list while
  `spec/surface/trial-retirement.md` still ran. The break was reverted.
