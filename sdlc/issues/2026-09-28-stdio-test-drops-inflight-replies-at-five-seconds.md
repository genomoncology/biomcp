# The stdio warn-once test drops in-flight replies at five seconds

Filed 2026-09-28 from the review of the 1255-1261 round, with the
mechanism read out of rmcp 1.7.0's source. The test failed in CI
twice (runs 36359461611 and 36364074552), both at ~5.02 s, and
passed four times in a row on the gate host and once alone in CI.

## Mechanism

`stdio_bad_fallback_starts_and_warns_once_across_tool_calls` closed
the server's stdin immediately after writing its five requests.
On end-of-input, rmcp 1.7.0 gives in-flight requests a five-second
grace window and then drops them
(`rmcp-1.7.0/src/service.rs:1052-1080`). The dropped replies closed
the read loop early, so the "all three tool calls answered"
assertion fired. A slow third call (the private-CA dial chain takes
about two seconds) pushes past the window under CI load. This is a
test bug, not a product bug: the shipped server honors the
protocol's shutdown rules.

## Fix

Keep stdin open until every expected reply arrives, then close it
deliberately so the server exits cleanly. Bound the diagnostic
stderr read the way the file's other stderr read is bounded. The
45-second total read budget stays as a backstop; it was never the
cause (both failures beat the old 10-second per-line limit too).

## Status

Fixed with the 2026-09-28 round (the drop moved after the read
loop). Owner: the biomcp queue. Revisit trigger: any future
stdio-contract test that closes stdin before awaiting replies, or
another five-second-capped failure in this test.
