---
base: f25b1000bb5d12d1cd1d447082b80b1a1ee7631f
head: 84462d78695643f33bf787f3d48241a9622447c7
---
The desired direction is a much slimmer routine gate: test that CLI commands route to the right internal request/plan objects, test that source clients construct the right HTTP requests and map fixture responses correctly, and keep only a tiny smoke layer for end-to-end confidence. Routine preflight should not call every public API or make unrelated live-source canaries block work.

Imported from March ticket 371. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/371-reset-biomcp-test-strategy-around-request-contracts
