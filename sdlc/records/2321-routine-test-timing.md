# Routine test timing

The outcome is to distinguish incorrect behavior and a hung test from slow correct work. The routine runners warn with each slow test's identity, duration and one-minute host load. Their outer hang guard remains a failure boundary. Product deadlines and their assertions remain product contracts.

The starting source is accepted integration `3cd3a0586bc65feab5e38a4580a5c7a6a7f992d8`. Only the approved ticket seed came from `b5aee25cdc194f48613b2d84fd83f6a8cd34cc07`.

Gator job `2321/1792541254149-2371357` ran the real nextest reporter over `PASS [   6.000s] probe test_slow_case` and inspected the active runner settings. It exited 1 and printed `SLOW_WARNING_PRESENT=False`, `RUST_HANG_TERMINATION_CONFIGURED=False`, and `PYTHON_WARNING_AT_FIVE_SECONDS=False`. This reproduces the reporting and configuration gaps without claiming a product defect or reproducing the cause of a historical intermittent failure.

The existing explicit-wait inventory contains 214 sites, including 159 deadlines. Review divides them by purpose:

- Test coordination includes subprocess completion, local fixture HTTP requests, startup, cancellation settlement, thread cleanup, marker polls, barriers and successful fixture reads. These use the routine hang guard rather than a separate short success deadline.
- Provider invocation expiry, retries, lock-budget expiry, cache freshness, cancellation timers and the MCP handshake timer exercise actual product policy. Their configured budgets, expected errors and elapsed-time assertions remain unchanged.
- Refused-target no-contact observations and partial-header socket controls assert an absence or framing boundary. Their observation windows remain explicit in the existing exception inventory. They do not require a correct operation to complete inside a short window.
- Deliberate slow providers, stalled workers and paused-clock timer controls remain stimuli. Existing sleep exceptions retain their purpose. This change does not refactor the legacy stimuli.
- Explicitly live checks, release/package builds and exhaustive long tests retain their own policy. The default routine hang guard does not impose a shorter limit on them.

The nextest default profile owns the warning period and hang-period count. The Python fixture helper and native contract helper derive coordination limits from that configuration. The Python collection hook supplies the guard only to routine tests without their own outer marker. Existing explicit routine markers use the same guard. The long nextest profile keeps warnings without termination. No product timeout setting changes.

The checks use planted slow completion, assertion failure and hang cases at the real runner boundary. A private shortened hang guard makes negative controls quick without weakening the configured routine policy. The required ordinary routine suite runs on both build platforms. Job logs retain exact commands and results; collection failures remain failures.
