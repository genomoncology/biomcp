---
flow: quickfix
priority: 9
---

# Make the provider-deadline finalization test clock deterministic

The provider deadline finalization regression now uses paused Tokio time. A
yield driver keeps virtual time live until cache publication arms; the test
then proves the deadline is initially live, advances it explicitly past the
boundary, and preserves the exact fail-closed cache and sanitized-error
assertions. The former real-time sleeps are no longer part of this proof.

## Evidence

Independent design and code reviews accepted the change. The focused test
passed 112 executions across implementation and review, the complete 10-test
provider-network module passed six runs, and Clippy passed
with warnings denied. `make lint`, `make test`, and `make spec` passed on the
frozen candidate under Node 22. The local `make test` run bounded Nextest to
eight workers after the shared 16-core host reproduced an unnamed pre-existing
suite concurrency failure at the default worker count; hosted CI retains the
default concurrency acceptance requirement.

## Boundary

Only the regression test changed. Production deadline, cache, provider, CLI,
MCP, and output behavior are unchanged.
