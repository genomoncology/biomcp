---
flow: build
priority: 10
---

# Adopt BioData eligibility JSON

BioMCP now delegates every populated `ClinicalTrialEligibility` JSON value to BioData. The field adapter passes the original JSON object to `ClinicalTrialEligibility::from_json_bytes`, uses `ClinicalTrialEligibility::to_json` for output, and retains BioMCP's outer missing and null behavior. BioMCP maps codec failures to one fixed message without source data.

The dependency and package boundary pin BioData 0.0.10 at `ae640f079314617583e22e099dc63d9c07f57b7c`. BioMCP removed its duplicate eligibility encoder, age reconstruction, code conversion, classification conversion, no-limit-rule constant, and owned wire types. A source-level ownership check prevents those rules from returning.

## Evidence

The focused red test failed against the old dependency revision and duplicate codec. The focused green run passed five eligibility tests and two package-boundary tests. The wider trial run passed 310 tests with two live-network tests skipped. Thirteen selected package-and-provider Python tests passed.

`make lint`, `make test`, `make spec`, and `make full-feature-check` passed. The routine Rust lane passed 3,297 tests with 30 skipped. The Python contract lane passed 912 tests with 3 skipped. Strict documentation, every routine specification suite, all-feature Clippy, six AlphaGenome tests, and the PNG, SVG, and terminal artifact smoke passed.

## Boundary

This change leaves trial responses, provider requests, eligibility search, Markdown presentation, MCP behavior, and reference output unchanged. It does not release or publish BioMCP.
