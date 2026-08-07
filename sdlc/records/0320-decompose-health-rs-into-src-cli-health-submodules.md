---
base: 0ba73a62209ca5718ed8a871ac169102ae9c5ab0
head: 0ec03fd22052ea85a3b15f242809bc8ae40ec1dc
---
`src/cli/health.rs` is 3,181 lines and currently interleaves the health source catalog, HTTP probe transport, local-data and cache checks, concurrency/timeout orchestration, and a 1,600+ line inline test block. `biomcp health` is also an operator-facing readiness surface with explicit local-source coverage contracts, so the file needs a clear architecture without changing any visible behavior.

Imported from March ticket 320. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/320-decompose-health-rs-into-src-cli-health-submodules
