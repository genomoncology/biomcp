---
base: c2dfaaa243ada65809f66c04c70a84ff0a9c6642
head: 7187a7303c72cf53c883e2180297004d2987c400
---
`biomcp study download` uses a DataHub client with only a connect timeout — `datahub_client(DATAHUB_CONNECT_TIMEOUT, None)` in `src/sources/cbioportal_download.rs` — and streams archive chunks in a loop with no idle/total/progress timeout. That choice is intentional to allow large archives, but it means a server that accepts the connection and stalls mid-download can hang `study download` indefinitely. There is no test pinning a stalled-stream failure mode and the public docs (`docs/reference/data-sources.md`) advertise a general 10s connect / 30s request timeout policy, so the DataHub exception is currently undocumented and unbounded.

Imported from March ticket 354. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/354-pin-cbioportal-study-download-stall-and-idle-timeout-policy-with-mock-server-contract-test
