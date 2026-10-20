---
flow: build
priority: 1
---
# Retain typed arm relationship failures

## Outcome

BioMCP preserves BioData's typed arm relationship failure whenever BioMCP constructs a `TrialDesign` directly or through a product conversion. Public CLI, JSON, raw MCP, and typed MCP errors remain sanitized. The design validator stops cloning arms and assignments only to validate them.

## Current facts

BioMCP main at `f583d4c307b962635ab5fefef8d878ad1ca06c5c` pins BioData `0.0.7` at `65f6af05720fdc0fbf630578be98ea34d77122d6`. BioData `0.0.8` at `a4c5bec98ab5185cc50bd6bc8c18f833b8d4097f` replaces the unit arm error with `ClinicalTrialArmRelationshipError`. Its five variants distinguish duplicate arm identity, duplicate intervention identity, a missing arm endpoint, a missing intervention endpoint, and a duplicate assignment. Each variant carries the relevant typed trial-local identities.

`TrialDesign::new` currently returns `Result<Self, ()>`. It clones the arm and assignment vectors into `ClinicalTrialArms::new`, then erases every validation failure. `product_design` and `product_nci_design` erase the same failure again as `BioMcpError::InternalProcessing`. The external architect identified this loss before eligibility adds more relationships.

ClinicalTrials.gov and NCI provider adapters already convert BioData provider projection failures into BioMCP's stable sanitized provider errors. BioData does not expose the inner relationship error from those provider errors. This ticket cannot recover information that was discarded before BioMCP receives it.

## Scope and decisions

Pin BioData `0.0.8` at `a4c5bec98ab5185cc50bd6bc8c18f833b8d4097f`.

Add a public non-exhaustive `TrialDesignError` with two cases: `SectionPresenceMismatch` and `InvalidRelationship(ClinicalTrialArmRelationshipError)`. Implement `Display`, `Error`, and a typed relationship accessor. Re-export it from the public path `biomcp_cli::error::TrialDesignError`. `TrialDesign::new` returns this error and calls `ClinicalTrialArms::validate` with borrowed slices. The section-presence check remains first because it describes BioMCP's product wire state rather than a BioData graph.

Add `BioMcpError::TrialDesign(TrialDesignError)`. Map failures from `product_design` and `product_nci_design` into that variant without flattening them. Preserve the public code `internal_processing`, the public message `Internal processing failed.`, the exit status, and the absence of provider and recovery fields. Expose the typed error only through Rust error inspection and the standard `Error::source` chain. Do not print trial-local identities or source values in a public response. Update `docs/reference/error-codes.md` so its complete error catalog names this variant and its stable public code.

Keep provider-origin ClinicalTrials.gov and NCI projection errors on their current paths and codes. In particular, known invalid provider projections remain sanitized provider errors. Do not change BioData's provider error API in this ticket. Do not add logging, a general error framework, or a test-only production seam.

Keep deserialization failures sanitized through the current JSON error boundary. Serde continues to convert constructor failures to a generic deserialization error. This ticket does not claim typed preservation through `TrialDesign::deserialize`. Focused tests may inspect `TrialDesignError`, but product responses must not reveal its variant or identities.

Add red-green tests for the section mismatch and all five relationship variants. Cover a missing intervention endpoint against a mismatched intervention collection. Add a compile-time external-path test that imports `biomcp_cli::error::TrialDesignError` and traverses `BioMcpError::source()` to the BioData relationship error. Exercise the new `BioMcpError::TrialDesign` variant directly through `code()`, `public_projection()`, `Display`, `exit_code()`, JSON error rendering, and the shared MCP error conversion. Valid provider values cannot reach this local failure, so do not add a manufactured provider failure or a test-only production seam. Keep the existing real CLI, raw MCP, and typed MCP provider contracts green. Keep successful ClinicalTrials.gov and NCI arm behavior unchanged, including the recorded NCI assignment count. Keep the package at or below 1,300 files. Keep net production Rust at or below 120 added nonblank lines.

## Acceptance

1. BioMCP pins BioData `0.0.8` at `a4c5bec98ab5185cc50bd6bc8c18f833b8d4097f`.
2. `TrialDesign::new` returns `TrialDesignError` and distinguishes `SectionPresenceMismatch` from all five exact BioData relationship variants.
3. Arm validation borrows the existing vectors through `ClinicalTrialArms::validate`; validation-only arm and assignment clones are gone.
4. `product_design` and `product_nci_design` preserve `TrialDesignError` inside `BioMcpError::TrialDesign`. The NCI branch remains unreachable through valid public BioData constructors, so code inspection and focused constructor tests cover the mapping without manufacturing malformed NCI data.
5. External Rust callers can import `biomcp_cli::error::TrialDesignError` and traverse the standard source chain from `BioMcpError::TrialDesign` to the exact BioData relationship error.
6. Direct public projections of `BioMcpError::TrialDesign` retain `internal_processing`, `Internal processing failed.`, exit status 1, and no provider or recovery fields. JSON and shared MCP error conversion expose no typed identity or source value. Existing real CLI, raw MCP, and typed MCP provider contracts remain green.
7. Provider-origin ClinicalTrials.gov and NCI invalid projections retain their current sanitized provider error codes and messages. The ticket makes no claim that BioMCP can recover a relationship error already discarded by BioData.
8. Existing successful ClinicalTrials.gov and NCI arm output remains green, including the recorded NCI assignment count. JSON deserialization remains strict and safe.
9. The complete error catalog documents `TrialDesign` and its stable `internal_processing` public code.
10. Net production Rust grows by no more than 120 nonblank lines. The package remains at or below 1,300 files.
11. An independent design review and an independent code review accept the result. Focused red-green evidence and `make lint`, `make test`, and `make spec` pass.

## Dependencies

BioData record 0101 supplies the typed relationship error and borrowed validator. BioMCP records 1175 and 1176 supply direct BioData arms and references. Both Factory channels remain paused. The manual subagent SDLC owns this work.

## Review

- Manual approval: Ian approved the clinical-trial delivery plan and directed implementation through the subagent SDLC without Factory.
- Design review: rejected once, then accepted after corrections. The independent reviewer required the ticket to exclude Serde from its typed-preservation claim, name `BioMcpError::TrialDesign`, name the public export path, update the complete error catalog, test public conversion surfaces directly, and avoid manufacturing an impossible NCI provider failure.
- Code review: accepted, remediated after the full lint gate, then accepted again. The first implementation preserved all five BioData relationship variants, the two-level Rust source chain, borrowed validation, generic Serde failures, unchanged provider mappings, and sanitized Display, JSON, and MCP projections. The full lint gate then rejected growth in three files with exact size ceilings. The remediation moved the trial error and JSON proof into the existing design module, moved the MCP proof into an existing test module, and removed duplicate tests. No ceiling changed and no package file was added. The same reviewer verified the revised complete diff and accepted it.
- Final verification: `make lint`, `make test`, and `make spec` passed on the accepted revision. The gate passed every lint and policy check, 3,274 Rust tests, 910 Python tests with three expected skips, strict documentation, and every offline specification group. Production Rust grows by forty-five nonblank lines. The package remains exactly 1,300 files.
