# spec-contracts omits two provider fixtures

Filed 2026-09-24 from a 2026-09-13 workspace report. Rechecked on main at `2d98594a`: the `spec-contracts` branch of `scripts/run-specs.sh` still starts only the article, study, and ClinicalTrials.gov fixtures.

- BioMCP 1.0 merged current BioMCP main through `181bc529`. The merge passed 60 focused author Rust tests, 7 focused author product and documentation tests, the source-package boundary test, and the dead-code and Rust-size quality ratchets.
- `make spec-contracts` then reported 160 passed blocks, 2 failed blocks, and 8 skipped blocks. Both failures are unchanged blocks in `spec/surface/mcp.md`: `Diagnostic Synonym Provenance Reaches Raw MCP` and `Variant filter evaluation is identical through raw and typed tools`.
- `scripts/run-specs.sh` is byte-identical between the dedicated 1.0 line and current BioMCP main. Its `spec-contracts` branch starts the article and study fixtures. It does not start `setup-provider-contract-spec-fixture.sh` or `setup-variant-identity-spec-fixture.sh`.
- The diagnostic block requires the provider-contract fixture for its GTR response. The variant block requires the variant-identity fixture for its MyVariant response. Normal `spec` mode starts both fixtures. Current-main ticket 1143 records a successful normal specification gate.
- Manual stdio initialization passed. Manual diagnostic CLI calls followed by stdio initialization also passed after the available specification fixtures were started. No incoming commit changed stdio startup or request execution.
- The two failures therefore measure missing `spec-contracts` fixture setup. They do not show a product regression in the rich author-papers merge. A BioMCP-owned correction can make the `spec-contracts` branch start and source the existing provider-contract and variant-identity fixtures with their existing cleanup paths.
