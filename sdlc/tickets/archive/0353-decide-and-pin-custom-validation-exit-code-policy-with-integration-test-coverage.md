---
flow: build
priority: 5
---
# Decide and pin custom-validation exit code policy with integration test coverage

Clap-generated parse/validation errors exit with code 2, while BioMCP's custom `BioMcpError::InvalidArgument` errors (e.g. `search gene` with no query, `search diagnostic` with no filters, `batch` with too many IDs) currently exit 1. The standard CLI convention is 0 = success, 1 = runtime failure, 2 = invalid usage; the inconsistency means script/agent callers cannot reliably distinguish a bad-usage error from an upstream/runtime failure. The 348 outside-in and code-review passes recommended either mapping `BioMcpError::InvalidArgument` to exit 2 at the CLI boundary in `src/main.rs` and pinning the mapping with integration tests, or explicitly documenting that BioMCP only uses exit 2 for clap parse errors.

Completed under March on 2026-04-29, as March ticket 353. Imported as history when BioMCP
moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/353-decide-and-pin-custom-validation-exit-code-policy-with-integration-test-coverage
