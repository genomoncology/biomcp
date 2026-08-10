---
flow: build
priority: 9
deps: ["0673", "0926", "0934", "0942"]
---
# Enforce no public network access in routine gates

The PubTator fixture conversion already landed. This ticket is the
fail-closed whole-suite boundary, after the source conversion and registry
reconciliation program finishes.

Its P9 label reflects the importance of the invariant, not early execution.
Dependencies deliberately make it the final routine-test enforcement gate
after lower-priority provider conversions and reconciliation have landed.

Offline-enforcement issue owned by this ticket:
sdlc/issues/output-footprint-corpus-is-nondeterministic.md

Ticket 0926 exclusively owns the separate defect in which disabled Semantic
Scholar still receives traffic. This ticket must not claim or delete ticket
0926's issue.

## Done when

On the canonical Linux gate, make test and make spec run in a network
namespace where public connections cannot succeed. Loopback and approved Unix
sockets remain available for local fixtures. Dependency fetching and all
builds happen before entering the isolated execution phase.

make verify remains the explicit live-smoke lane and is not run inside this
isolation.

## Enforcement design

Use bubblewrap or an equivalently fail-closed Linux network namespace. Do not
depend on proxy variables: several production clients deliberately use
no_proxy and direct client builders.

- The blocking CI image installs the namespace tool; missing support fails the
  enforcement job rather than silently skipping.
- A negative control attempts DNS and a direct public connection and must fail.
- A positive control proves an ephemeral loopback HTTP fixture and a bounded
  Unix socket still work.
- The runner reports which isolation mechanism is active.
- Non-Linux local development may use an explicit unsupported message, but
  cannot be the authoritative enforcement result.

Make output-footprint fixtures deterministic before running under isolation.
Delete the named issue file when the ticket lands.

## Authorized test changes

Design commits may restate make targets, test/spec runner wrappers, CI setup,
the output-footprint fixture contract, and isolation tests. Do not change
provider behavior or weaken a biomedical assertion to make the isolated gate
green.

The src line ceiling may not rise.
