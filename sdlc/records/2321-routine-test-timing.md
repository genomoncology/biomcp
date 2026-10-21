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

## Affected checks

Gator job `2321/1792542000415-2450696` passed the six integrated sequential-worker controls, Ruff and warning-denied all-target Clippy with the installed supported Rust 1.95 toolchain. The prior default compiler was newer and rejected unchanged `fetch_update` calls; those unrelated product calls remain unchanged. The same job's real Rust timing control found that nextest's changing progress prefix caused duplicate warnings. The regression table now includes that actual output format.

Gator job `2321/1792542191242-2454147` passed in 17,313 ms. Real Rust and Python cases each completed after six seconds, passed, and produced exactly one warning containing their identity, time and one-minute load. Hung cases failed under a private shortened runner guard. The long Rust profile passed past that private routine guard, proving it did not inherit termination. The reporter regression table and Ruff also passed. The job used published starting source `3cd3a0586bc65feab5e38a4580a5c7a6a7f992d8` with recorded payload SHA-256 `0609568a2ba0306b51407bf5ac157230b6892f6ee092bf753a7ba8f407ccacc6`; it is affected-check evidence for that payload, not an ordinary gate on a published final source.

The ordinary Gator job `1792542852708-2495311` passed Rust and 1,840 Python cases but failed the nested offline namespace probe. Focused `j157` confirmed that its caller was already enclosed. Direct diagnostic `j159` retained the actual bubblewrap message, `No permissions to create a new namespace`. The existing enclosed-profile exception matched the same diagnostic without `a`. The correction accepts those two explicit wordings only when already enclosed. It changes no privilege, network, ownership or transport assertion and accepts no other sandbox failure. M5's ordinary job `1792542787758-88716` passed the preceding source in 866,708 ms; that receipt is not a claim about a later source.
