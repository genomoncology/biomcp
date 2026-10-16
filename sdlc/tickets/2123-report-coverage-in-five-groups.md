# 2123 — Report coverage in five groups

Status: OPEN.

Milestone: 0.9.3

## Outcome

The coverage block of `tools/measure-0-9-3-baseline` reports product code in five groups, reached from outside tests, reached only by internal tests, reached by both, unobserved, and failed or missing collections, over one stated denominator, from the profiles it already collects.

## Evidence

- 2116 audit, 2026-10-16: the block prints three line percentages (outside-in, unit, combined). Nothing shows internal-only, both, unobserved, or failed collections, so 2081's deletions and 2082's after-record need a per-file method by hand.
- The shared build-efficiency rules, 2026-10-16, ask for these five groups and say overlap only nominates tests to remove.
- 2117 landed the failed-suite marking and the stated denominator; this ticket builds on it.

- Starts from: `tools/measure-0-9-3-baseline` coverage section.
- Decisions:
  - Derive the groups from the outside-in and unit reports the script already merges, per line, with no extra test runs.
  - Print each group as a line count and a share of the denominator; a group fed by a failed or missing collection prints incomplete, as 2117 does.
- Changes: claims `tools/measure-0-9-3-baseline` `CHANGELOG.md`.
- Proof: one run of the coverage section on the Linux gate host; the four line groups sum to the denominator, and the record shows them in this ticket.
- Order: after 2117 (landed) and 2120; before 2082 runs.
