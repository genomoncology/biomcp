# 2116 — Run the 0.9.3 build efficiency audit

Status: OPEN.

Milestone: 0.9.3

## Outcome

The ticket holds a measured audit of BioMCP's build and test cost after the 0.9.3 speed work already landed, and a list of findings, each with its measured cost now and the expected cost after a fix, ready to file as one ticket each.

## Evidence

- Asked for on 2026-10-16: every project audits its build and test efficiency once per release against shared rules (development, ticket-close, release, coverage and performance runs kept apart; shared output built once; named build modes; coverage reported in five groups; a fixed set of development benchmarks).
- 2069 holds the 0.9.3 before numbers at v0.9.2. Since then 2070, 2075, 2083, 2086, 2071, 2101, 2102, 2103 and 2105 landed; 2076, 2079, 2080, 2081, 2094 and 2095 are open.

- Starts from: `tools/measure-0-9-3-baseline`, `sdlc/records/2069-0-9-3-baseline.md`, `.github/workflows/ci.yml`, `Makefile`, `.config/nextest.toml`, `Cargo.toml`.
- Work, on the Linux gate host through the job runner, at one main revision:
  1. Machine time: total the last two weeks of gate-host job records and green CI runs; split into routine, release and other runs, waiting against running, and repeat runs on identical code.
  2. The twenty costliest tests or steps from the routine suite and CI; for the top five, setup time against work time.
  3. Shared output built more than once per revision: dev build, nextest archive, spec binaries, Python lane, CI jobs.
  4. Every build flag set in use (Makefile modes, CI jobs, spec profile, coverage), and whether each keeps the compiler cache and the fast linker.
  5. The coverage block in `tools/measure-0-9-3-baseline` against the rules: one stated denominator, five groups, a failed collection marked incomplete (2117 owns that last fix).
  6. The development benchmarks: a fresh build with a warm cache, a no-change rebuild, a small code edit, a docs-only edit, the routine suite and the release suite, each with wall time, peak memory and target growth; several samples, raw samples kept in the ticket.
- Decisions: findings already owned by an open ticket name that ticket and add only new numbers; this ticket fixes nothing itself.
- Changes: claims this ticket only.
- Proof: the six sections above written into this ticket with commands and raw numbers; a findings list, each with cost now and expected cost after.
- Order: safe alongside anything; run it once, at one main revision.
