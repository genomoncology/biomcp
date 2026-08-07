---
flow: build
priority: 6
---
# Pin cBioPortal study download stall and idle timeout policy with mock-server contract test

`biomcp study download` uses a DataHub client with only a connect timeout — `datahub_client(DATAHUB_CONNECT_TIMEOUT, None)` in `src/sources/cbioportal_download.rs` — and streams archive chunks in a loop with no idle/total/progress timeout. That choice is intentional to allow large archives, but it means a server that accepts the connection and stalls mid-download can hang `study download` indefinitely. There is no test pinning a stalled-stream failure mode and the public docs (`docs/reference/data-sources.md`) advertise a general 10s connect / 30s request timeout policy, so the DataHub exception is currently undocumented and unbounded.

Completed under March on 2026-04-29, as March ticket 354. Imported as history when this
repo moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/354-pin-cbioportal-study-download-stall-and-idle-timeout-policy-with-mock-server-contract-test
