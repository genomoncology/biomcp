---
base: eea064ff7873d95daa94d653205e870ec80ccf29
head: a32ae00aa4e4b7aa2320623c8d2d8b2954926868
---
A live `discover BRCA1` MCP full-contract test lives in the **blocking `make test` gate** (the `focused` / `full-blocking` validation profiles). Because it hits real external MCP services it is non-deterministic, and when it flakes it **false-fails the `03-code` / `05-verify` step of whatever build ticket is running** — even when that ticket's own changes are sound. On 2026-07-16→17 it bounced ticket 582 three times and held a team-wide pause for hours, idling the queue behind it, while 582's own implementation was complete, committed, and green on lint/spec/native tests. Ticket 585 fixed one instance (the OLS4 stub connection budget); this ticket is the durable fix so the class stops eating cycles. A blocking gate that depends on live external services is non-deterministic by construction.

Imported from March ticket 591. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/591-move-live-full-contract-tests-out-of-the-blocking-make-test-gate-into-verify
