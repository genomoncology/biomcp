---
flow: build
priority: 6
deps: ["0686"]
---
# The server-starting run wrappers adopt the supervisor

Ticket 0686 landed one owner-identity-aware fixture supervisor. Reuse it for
exactly these five wrappers:

- run-article-semanticscholar-source-search.sh
- run-clingen-erepo-fixture.sh
- run-section-outcome-mcp.sh
- run-variant-article-entity-fixture.sh
- run-variant-article-identity-fixture.sh

## Done when

Killing each wrapper's real exported owner with SIGKILL cannot leave its
server, process group, socket, port, or owned temporary root behind. Normal
exit and timeout cleanup also work. Recovery validates fixture kind,
worktree/root prefix, owner token, PID start identity, and process group
before signaling.

Generalize 0686's disease-specific marker/recovery dispatch only as needed to
reuse the same supervisor. Do not create wrapper-specific supervisors.

## Authorized test changes

Design commits may restate the five wrappers, shared supervisor scripts, and
their lifecycle tests. Every wrapper gets a real exported-owner path plus
SIGKILL proof; a source-text assertion is not sufficient.

The src line ceiling may not rise.
