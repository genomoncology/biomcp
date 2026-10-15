# Delete tests boldly and split the suites

October 14, 2026. Ruled by Ian.

## The ruling

Ian, on the 0.9.3 test work:

> Remember, we can always add tests back. The problem is we never delete them, and most of them are crap.

On ticket-branch CI at about 25 minutes:

> 25 minutes is probably too long even for CI.

Asked to choose between a cautious plan and a more aggressive one, he approved the aggressive one: "yes, if you think that's aggressive enough." His earlier ruling stands that program speed matters more than build speed.

## What it means

1. **Routine tests keep only four kinds.** Behavior tests through the real entry point, edge-case tables, contract checks on what others depend on, and regression tests that failed before their fix. Everything else goes, boldly, in small batches that each land on their own. A deleted test comes back when a real need shows up. Coverage is reported with each batch and is not a gate. Mutation testing is an optional tool for finding the next outside-in test to write, not a gate on deletion. BioMCP refuses rather than guesses, and the tests that pin those refusals and the fixed regressions are regression tests: they stay.
2. **Two suites.** Ticket-branch CI runs the routine suite only, with a target of about 10 minutes wall time. The all-features build, Windows, the stress lane and release-panic run on pushes to `main` and in the release workflow. A ticket that changes code only those jobs build runs `make full-feature-check` itself before landing. A release still needs a finished green CI run on the tagged commit, as Ian ruled on 2026-09-29.
3. **One full check per ticket.** Once ticket-branch CI is at about 10 minutes, a developer runs only the tests for what changed, and CI is the ticket's one full gate. This overrides, for BioMCP, the shared rule to run the gates once before landing. The ticket that brings CI to about 10 minutes writes the rule into `AGENTS.md`.
4. **Hard targets for 0.9.3**, measured against the 2069 baseline record: ticket-branch CI about 10 minutes; `make test` about 3 minutes; `make spec` about 3 minutes; every routine test a few seconds at most, enforced by the test runners once the slow tests have injected clocks. The share of test code is measured with each deletion batch, and its target is set after the first batch.
5. **Order.** The test cleanup moves ahead of the remaining build and dependency work. The run-time speed tickets and the product fixes run in a parallel lane beside it, not behind it.

## Options considered

- **The cautious plan.** Delete unit tests only after the replay server lands, keep combined coverage within a few points of 84%, and require mutation evidence before each deletion. It protects every covered line, and it is slow: deletion waits on the largest ticket in the milestone, and mutation runs on a crate this size take hours per module.
- **Bold deletion (chosen).** Delete what is not one of the four kinds, in small batches, and add a test back when a real need shows up. It risks losing a check nobody noticed. Small batches, the kept regression tests and CI on every batch limit that risk, and a missing test is cheap to restore.

## What this replaces

- The 2081 bar that combined coverage stays within a few points of 84%, and 2081 waiting for 2079.
- The 2075 PM decision of 2026-10-14 that only release-panic moves to main-only and that full-features and windows-contracts stay on every push, with its 22-minute target.
- "A few minutes" as the spec-page target in 2080.
- The 0.9.3 order in `sdlc/plan.md` that put the run-time tickets ahead of the test cleanup and the test cleanup last.

## Amendment, 2026-10-14

The tickets state the targets as checkable numbers: ticket-branch CI median 11 minutes or less and `make test` 200 s or less (2094), `make spec` 200 s or less (2080), and no routine test over 10 s (2095). Those numbers are the enforceable form of the targets in rule 4.
