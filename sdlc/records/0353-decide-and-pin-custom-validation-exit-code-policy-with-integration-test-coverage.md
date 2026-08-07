---
base: eece35e173ee896de7872a25aa6d5714ca845a09
head: 3895653764d80b332af1582e2101fa701bc22457
---
Clap-generated parse/validation errors exit with code 2, while BioMCP's custom `BioMcpError::InvalidArgument` errors (e.g. `search gene` with no query, `search diagnostic` with no filters, `batch` with too many IDs) currently exit 1. The standard CLI convention is 0 = success, 1 = runtime failure, 2 = invalid usage; the inconsistency means script/agent callers cannot reliably distinguish a bad-usage error from an upstream/runtime failure. The 348 outside-in and code-review passes recommended either mapping `BioMcpError::InvalidArgument` to exit 2 at the CLI boundary in `src/main.rs` and pinning the mapping with integration tests, or explicitly documenting that BioMCP only uses exit 2 for clap parse errors.

Imported from March ticket 353. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/353-decide-and-pin-custom-validation-exit-code-policy-with-integration-test-coverage
