# Live verify has unrelated failing diagnostics

Severity: should-fix.

This is live-verify-lane work. `make verify` is deliberately not a
gate rung here (`sdlc/planning/verify-lane.md`), so this cannot fail
a flight and should not outrank gate-lane work when it is triaged.

Carried over from March, where it was raised against ticket 612
on 2026-07-24 and left open. The text
below is as filed.
## Summary

`make verify` cannot provide a green aggregate result because unrelated live diagnostics fail or time out, even though ticket 612's two CAR assertions pass directly with the release binary.

## Detail

On 2026-07-24, the live suite failed article recommendation/citation assertions, the G5 v2 readiness diagnostic timed out, and the discover-code labels diagnostic lacked expected SNOMEDCT and ICD10CM source rows. `spec/entity/clingen-car-live.md` passed both assertions directly with `BIOMCP_BIN=target/release/biomcp mustmatch test spec/entity/clingen-car-live.md --lang bash -v`.

## Suggested action

Investigate each failing upstream diagnostic and either repair its runtime behavior or make the verify harness report independently attributable source failures. Improved-test destination: verify-group.

## Audit update — 2026-07-24

- The discover label failure no longer reproduces on current main: a no-cache direct
  command returned both `SNOMEDCT` and `ICD10CM` with no errors. Issue 601 is closed with
  the exact commit/binary proof.
- An independent direct G5 v2 run again reported 7/7 resolved identities, exact routes,
  route-tied aliases, source status, and terminal state. Its prior timeout is not a
  current identity-contract failure, although honest live incompleteness remains visible.
- The legacy seven-variant recall gate is independently still red and remains release
  blocking under issue 605.
- Recommendation/citation diagnostics require one current attributed rerun before their
  disposition is changed.

Ticket 623 owns the final grouped reconciliation and must not weaken any threshold or
convert a required provider outage into healthy emptiness.
