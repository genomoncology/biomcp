---
flow: quickfix
priority: 10
---
# The suite survives the traced observation

The channel auto-paused on 2026-08-29 after seven faults on the
1084 adoption: the propagated green-main proof runs the gates
under an execve tracer, and the traced full suite fails while the
plain gate stays green. Evidence log 102 shows the shape — 3033
of 3035 tests pass, but the whole run saturates under the
tracer's slowdown (22 tests marked SLOW, several passing only at
210 to 225 seconds), and two tests cross an internal deadline and
fail:

- `entities::variant::get::tests::population_request_requires_a_grch38_genomic_coordinate`
  fails at src/entities/variant/get/tests.rs:504 after 225
  seconds. It asserts population status `"data"` against a local
  fixture server and receives `"unavailable"` — a client-side
  timeout dressed as an answer.
- `human_mcp_command_dispatches_to_provider_once` fails at
  tests/rmcp_client_contract.rs:172 after 97 seconds. It asserts
  `is_error == Some(false)` against local stubs and receives
  `Some(true)`.

Both tests pass plain in about 12 seconds each, verified
2026-08-29 on the live checkout at c0ad5a74. Two sibling repos
fixed this exact class this week by widening test-side timing
budgets.

Required behavior: the full test gate passes under the proof's
own wrapper, with no behavioral assertion weakened.

## Reproduction

```
strace -f -qq -e trace=execve -s 0 -xx -o /dev/null -- sh sdlc/scripts/test
```

Done, observably:

- The reproduction command exits zero on the repaired tree.
- `sh sdlc/scripts/test` and `sh sdlc/scripts/lint` exit zero,
  plain.

Boundary: test-side timing budgets only — the deadlines, injected
timeouts, or stub response windows the two named tests depend on,
and any sibling tests the reproduction proves carry the same
assumption. Do not change production timeout behavior, the MCP
dispatch code, the population lookup code, or what any test
asserts about behavior. If the failing deadline turns out to live
in production code with no test-side lever, refuse and name the
code path — that is a build ticket, not a quickfix.
