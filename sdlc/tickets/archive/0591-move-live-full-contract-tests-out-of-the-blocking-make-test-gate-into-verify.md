---
flow: quickfix
priority: 11
---
# Move live full-contract tests out of the blocking make-test gate into verify

A live `discover BRCA1` MCP full-contract test lives in the **blocking `make test` gate** (the `focused` / `full-blocking` validation profiles). Because it hits real external MCP services it is non-deterministic, and when it flakes it **false-fails the `03-code` / `05-verify` step of whatever build ticket is running** — even when that ticket's own changes are sound. On 2026-07-16→17 it bounced ticket 582 three times and held a team-wide pause for hours, idling the queue behind it, while 582's own implementation was complete, committed, and green on lint/spec/native tests. Ticket 585 fixed one instance (the OLS4 stub connection budget); this ticket is the durable fix so the class stops eating cycles. A blocking gate that depends on live external services is non-deterministic by construction.

Completed under March on 2026-07-18, as March ticket 591. Imported as history when BioMCP
moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/591-move-live-full-contract-tests-out-of-the-blocking-make-test-gate-into-verify
