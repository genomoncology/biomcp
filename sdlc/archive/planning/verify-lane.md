# The live verify lane — outside the factory, on purpose

`make verify` exercises BioMCP against the real external services. It is
**not** a gate rung. The factory's ladder is `lint` → `test` → `spec`, and
all three are deterministic: same commit in, same answer out.

## Why it is not a rung

An unattended flight must not be judged on someone else's uptime. If
`make verify` were a gate, a ticket would fail because MyVariant was slow,
because an API key expired, or because a rate limit was hit — none of which
the agent caused and none of which the agent can fix. Those failures would
land on the ticket's record as if the work were wrong.

Keeping it out of the ladder does not retire it. Live behavior still has to
be checked; a human decides when, and reads the result knowing what a
network failure looks like.

## What it covers

Sixteen live spec pages that the routine lane deliberately skips, plus the
NIH RePORTER pages behind a separate soft-failure wrapper. Together they are
the assertions that talk to CT.gov, MyVariant, MyGene, MyDisease, PubMed,
DDInter, OLS4, and the rest. `Makefile` is the authority on the exact list —
see the `verify` target and the `verify` case in `scripts/run-specs.sh`.

It also runs the release binary, so it builds `--release --locked` first.

## How to run it

From the repo root, with credentials in the environment — never in a file
in this repo:

```sh
make verify
```

Keys the live sources read, when present: `NCBI_API_KEY`, `NCI_API_KEY`,
`UMLS_API_KEY`, `OPENFDA_API_KEY`, `DISGENET_API_KEY`,
`ALPHAGENOME_API_KEY`. A missing key does not always fail loudly — some
pages degrade or skip — so read the output rather than only the exit code.

## When to run it

- Before a release, always. `make release-gate` and `make
  release-live-smoke` are the release-shaped entry points.
- After any change to a source client, a request shape, or a captured
  fixture — the whole point of the captured-response work is that the
  routine lane stops noticing when a provider changes, so something has
  to keep noticing.
- On a schedule, if someone sets one up. Nothing schedules it today.

## The honest gap

Nothing currently forces this to run. If it goes unrun for a long stretch,
a provider can change shape and the factory will stay green through it.
That is the accepted cost of not gating unattended work on live services,
and it is worth writing down rather than discovering later.
