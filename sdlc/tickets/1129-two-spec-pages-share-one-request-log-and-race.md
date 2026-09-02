---
flow: build
priority: 9
---

# Two spec pages share one request log and race, so the spec gate fails at random

`make spec` failed on ticket 1128 in `spec/entity/trial-numeric-filters.md`, case "Zero distance is rejected before ClinicalTrials.gov work", with `expected exit 2 but observed exit 1`. An immediate rerun on the unchanged candidate passed. The attempt was refunded, its work preserved in commit `e60dd3d2`, and the channel backed off.

The failure is a race between two spec pages over one file. Nothing about the candidate was wrong.

## The race

The provider fixture is set up once for the whole run, at `scripts/run-specs.sh:217`:

```
bash spec/fixtures/setup-ctgov-intervention-alias-spec-fixture.sh "$ROOT"
```

That script creates a single fixture root and a single log inside it, at `spec/fixtures/setup-ctgov-intervention-alias-spec-fixture.sh:22` and `:27`, then exports the path to every worker at `:534`. The fixture server appends every request to that one file, at `:399` and `:490`.

Spec pages then run in parallel workers, from `scripts/run-specs.sh:134` onward.

Two pages use that one file in incompatible ways.

`spec/entity/trial-numeric-filters.md` truncates it and requires it to stay empty:

```bash
: >"$BIOMCP_CTGOV_INTERVENTION_ALIAS_REQUEST_LOG"
...
test ! -s "$BIOMCP_CTGOV_INTERVENTION_ALIAS_REQUEST_LOG"
```

`spec/entity/trial.md:280-282` drives requests through the same fixture and greps that same file for the entries its own requests produce.

Run concurrently, each breaks the other. A request from `trial.md` landing between the truncate and the check makes `test ! -s` fail, and because the block runs under `set -e` the script exits 1 before reaching `exit "$status"` on line 22. That is exactly the observed `expected exit 2 but observed exit 1`. In the other direction, the truncate deletes entries `trial.md` is about to grep for.

The case's own assertion is fine. `--distance=0` really is rejected before any provider call. The page fails on a fact about a neighbouring page instead.

## Required behavior

A spec page's result depends only on the behavior it tests.

No two pages share a mutable file. A page that asserts "no request was made" observes only its own requests.

## Done, observably

- `make spec` passes repeatedly with pages running in parallel. Run it enough times to be evidence rather than one lucky pass, and say in the record how many.
- `trial-numeric-filters.md` still proves that `--distance=0` exits 2 and makes no provider request, and `trial.md` still proves its three request shapes reach the fixture. Neither loses coverage.
- A test or check fails if a future page takes a shared mutable path from the fixture environment. The rule holds mechanically rather than by memory.
- `tests/surface/test_parallel_isolation_contract.py` covers this case, or the ticket says why that file is the wrong home for it.

## Boundary

Do not serialize the spec run to hide the race. Parallel execution is the point of the runner, and a slower gate is a worse gate.

Do not weaken either assertion. Dropping `test ! -s` would remove the only proof that validation happens before provider work, which is the whole subject of the page.

Do not change the CLI's behavior. Nothing here says the tool is wrong.

Whether each page gets its own fixture root, or the log is partitioned per worker, or the page filters the log to its own requests, is a design choice. Any of them satisfies this ticket.

## Why this matters beyond one flake

This is the third non-deterministic gate failure on this channel today, after the OpenFDA regulatory overrun in 1127 and the `$TMP` ceiling faults on 1092. Each one refunds its attempt, so the board shows a ticket that keeps looking ready and only the fault detail says why. A flaky gate is not a small cost. It stops the queue and hides the reason.
