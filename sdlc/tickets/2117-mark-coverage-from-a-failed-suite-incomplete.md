# 2117 — Mark coverage from a failed suite incomplete

Status: OPEN.

Milestone: 0.9.3

## Outcome

The coverage block of `tools/measure-0-9-3-baseline` prints a coverage set as incomplete, with the failed suite named and no percentage, whenever any suite feeding it failed. It states its denominator once.

## Evidence

- The 2069 baseline record prints outside-in 52.66 %, unit 78.30 % and combined 86.26 % line coverage, while its own notes say the instrumented integration suite ran 135 tests with 1 failing and the Python lane lost 26 tests under instrumentation. Those percentages read as complete numbers.
- The shared build-efficiency rules, 2026-10-16: a collection with a failed suite is incomplete; say so and never print a bare percentage for it. Use one denominator, product code without tests or generated code, and state it.
- `tools/measure-0-9-3-baseline:450-524` runs the instrumented suites through `timedsh` and prints `coverage_<set>_lines_pct` from the report alone.

- Starts from: the lines above.
- Decisions:
  - Record each instrumented suite's exit status and test counts. A report set fed by any failed or short suite prints `RESULT coverage_<set>_lines_pct = incomplete (<suite>: <n> failed)` instead of a number.
  - Print the denominator rule once in the block: product code, with test files and generated code removed, and inline test modules counted or not, as the script actually does.
- Changes: claims `tools/measure-0-9-3-baseline` `CHANGELOG.md`.
- Proof: a run of the coverage section with one suite forced to fail prints incomplete for every set it feeds and a number for the rest; a clean run prints numbers. Record both outputs in the review.
- Order: shares `tools/measure-0-9-3-baseline` with 2082; land before 2082 runs.
