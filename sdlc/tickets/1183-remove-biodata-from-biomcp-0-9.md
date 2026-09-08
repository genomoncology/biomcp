---
flow: build
priority: 10
deps: []
---
# Remove BioData from BioMCP 0.9

## Goal

BioMCP 0.9 builds, tests, packages, and serves its current clinical-trial CLI and MCP contracts without a BioData dependency, checkout, repository, release, credential, or runtime concept. BioMCP locally owns the narrow strict trial values, request plans, and provider adapters its product uses.

## Current evidence

This urgent release-boundary defect is present on `origin/main` at `a694f1dc68c548a1ab7113de20ea6b08772e6d14` (`0.9.0-dev.6`). `Cargo.toml` has a direct exact Git dependency on `https://github.com/genomoncology/biodata` at `cfafc69d27c9a2fc74909f21692a418a8b17db83` and a `[package.metadata.biodata-development]` block that defers extracted-package compilation. `Cargo.lock` records `biodata 0.0.11`; `cargo tree --locked -i biodata` reports the direct edge:

```text
biomcp-cli
└── biodata (private exact Git revision)
    ├── quick-xml
    ├── serde / serde_json
    └── sha2
```

The required completed graph is:

```text
biomcp-cli
├── local trial values and strict JSON codecs
├── local CTGov detail plan and response adapter
├── local NCI CTS detail plan and response adapter
└── existing public dependencies only

BioData: no node, edge, checkout, generated copy, or runtime handoff
```

The dependency is not isolated. Production imports occur in `src/entities/trial/design.rs`, `eligibility.rs`, `get.rs`, and `mod.rs`; `src/render/markdown/trial.rs`; and `src/sources/clinicaltrials.rs` and `nci_cts.rs`. BioData supplies arm/intervention identities and relationship validation, eligibility and reference values/codecs, request-aware section states, exact CTGov and NCI detail plans, strict bounded response parsing, identity/cardinality checks, and sanitized adapter error codes. Rust and Python tests construct or assert those types and handoffs. `tests/test_source_package_boundary.py` requires the Git revision and compile deferral instead of proving an extracted package compiles.

The exact current test/comment handoffs are `src/entities/trial/get/tests.rs`, `src/entities/trial/test_support.rs`, `src/mcp/shell/typed_get_tests.rs`, `src/render/markdown/root_tests.rs`, `src/render/markdown/trial/tests.rs`, `src/sources/clinicaltrials/tests/{construction,parsing}.rs`, `src/sources/nci_cts/tests/{construction,live,parsing}.rs`, `src/transform/trial/tests.rs`, `src/transform/trial/tests/ticket_1132.rs`, `tests/json_error_contract.rs`, `tests/test_nci_filter_transport.py`, and `tests/test_source_package_boundary.py`. Current public prose coupling is exactly `docs/user-guide/cli-reference.md` and `docs/user-guide/trial.md`; no current `spec/**` or `skills/**` file names BioData, but those shipped surfaces must remain protected from reintroduction.

History shows why a raw revert is wrong. `e3f108d0` established strict CTGov reference retrieval; `554fc137` replaced the lenient NCI `trials/{id}`/untyped reader with the current field-selected `trials` request and strict envelope, identity, and cardinality rules; `c988e43d` established identity-bearing arm assignments; `aa6a7fe6` changed stored reference values; `aafb52b7` established rich cross-provider eligibility and default-detail behavior; `ce1d4748` moved strict eligibility JSON to BioData; and `7e85db24`/`e8f02aa8` established structured reference JSON and current Markdown fallbacks. The pre-`554fc137` NCI reader and the retired scalar `reference_type` output do not meet today's product contract.

Current public Rust exposure is narrower than the CLI/MCP surface but is not zero-coupled: `BioMcpError` publicly re-exports `TrialDesignError`, whose relationship variant and accessor expose `biodata::ClinicalTrialArmRelationshipError`. Exact Rust source compatibility for that foreign type and zero dependency are mutually exclusive.

Current forward-looking coupling also remains in the two user-guide statements, `sdlc/planning/clinical-trial-capabilities.md`, stale active ticket copies 1180 and 1181, and ticket 1182. Completed `sdlc/records/**`, archived tickets, historical planning notes, and the fixture authorship/license receipt in `testdata/sources/capture-receipts.json` are historical evidence, not executable coupling.

## Required behavior

Implement a clean BioMCP-local boundary. Do not vendor, subtree, copy wholesale, rename, publish, or add a path/patch dependency on BioData. Do not require a sibling checkout, Git access, generated BioData source, environment variable, feature flag, CI secret, token, SSH key, or registry credential. Do not publish BioData or coordinate a BioData release. Historical commits and provider-backed BioMCP fixtures may be used as evidence; the implementation must remain the smallest product-owned code that satisfies BioMCP's contracts.

Locally own these values and invariants in the trial entity layer:

- source authority/code plus nullable display, vocabulary version, and recognized meaning, rejecting empty present values;
- positive portable arm, intervention, and eligibility-criterion identities in `1..=9_007_199_254_740_991`;
- interventions, arms, arm/intervention assignments, and typed duplicate/missing-endpoint relationship errors;
- request-aware `Present`, `Absent`, `NotRequested`, and `Unavailable` section states where the adapters require them;
- eligibility registry text, age range/bounds, source duration, sex codes, healthy-subject state, ordered criteria, classification, and the NCI `999 Years` source-stated-no-limit rule;
- references with exactly nullable `pmid`, `citation`, and `source_type`, where a populated source type has exactly the five code members.

Keep `TrialDesign` as the owning graph boundary. Replace its foreign relationship error with a local error that retains the current variant names, fields, equality/copy behavior, data-free `Display`, `Error::source`, and accessor semantics. Re-export the reachable local error beside `TrialDesignError`. This is the one accepted pre-1.0 Rust source change: callers naming or downcasting the foreign BioData type must migrate to BioMCP's local type. No successful CLI, MCP, JSON, Markdown, or shell contract may change.

Implement local strict value codecs. Eligibility keeps all five required members; references keep all three required members and all five required nested code members, including explicit nulls. Preserve missing/null/empty distinctions, Unicode, order, source values without storage-time trimming, duplicate-member rejection at the root and nested levels, unknown/missing/wrongly typed member rejection, portable identity bounds, duplicate criterion rejection, exact duration source quantity/unit/bound validation, and fixed data-free errors `invalid clinical trial eligibility` and `invalid clinical trial reference`. Markdown alone may trim for display and must retain the PMID-only behavior from `e8f02aa8` and all current safe fallbacks.

Implement only the local provider adapters BioMCP consumes. Do not recreate BioData capture reports, conversion reports, generic cross-product trial projections, legacy XML, or unrelated values.

The CTGov adapter must preserve the exact normalized `studies/{NCT ID}` path; sorted, deduplicated `fields` query; always-requested base/intervention fields; default-detail eligibility; explicit/all section composition; eligibility-triggered document fields; request-aware references, arms, and eligibility; exact returned-identity validation; typed arm occurrence assignments; and current reference and eligibility projection. It must continue to parse the same untouched response bytes used by the product's ordinary study model.

The NCI adapter must preserve `GET trials`, `size=1`, normalized `nct_id`, every ordered repeated `include` pair, the single `X-API-KEY` header, eligibility inclusion only when requested/defaulted, and `brief_summary`. It must preserve status-before-body handling, zero/one/multiple-row behavior, exact raw returned-identity validation, rejection of the old lenient shapes, required field meanings, exact whole portable accrual conversion, nested arm/intervention occurrence identities, stable numeric `display_order` eligibility ordering, classifications, and current product projection. The API key must never enter a credential-free plan, cache key, model, error, log, fixture, or output.

Retain finite defense in depth. The transport already caps ordinary provider bodies at 8 MiB; local strict decoding must also enforce inclusive maxima of 8 MiB input, depth 128, 4 Mi decoded Unicode scalars in one string, 1,000,000 elements in one collection, 16,384 members in one object including duplicates, and 1,000,000 normalized events. Check HTTP status before JSON validation. Preserve adapter codes `malformed_json`, `unsupported_json`, `json_resource_limit`, `invalid_projection`, and `identity_mismatch`, plus NCI `not_found` and `unexpected_row_count`. Only `json_resource_limit` maps to narrow-request recovery; other validation failures retain retry-source recovery. Errors and logs must not contain response bytes, rejected values, decoder paths, trial/contact/criterion sentinels, credentials, or local paths.

Remove the runtime phrase `BioData response validation failed` in favor of a provider-neutral, data-free validation message while preserving the structured BioMCP error code, source, recovery action, exit status, stdout-only JSON discipline, and no-partial-output behavior. This intentional human-message cleanup is required by zero runtime coupling; successful public output remains byte-compatible.

## Zero-coupling surfaces and ratchet

Remove the dependency, lockfile package/edge, compile-deferral metadata, imports, foreign type names, helper names, runtime messages, comments, and ownership assertions from these active surfaces:

- `Cargo.toml`, `Cargo.lock`, `Makefile`, `.github/**`, `scripts/**`, `tools/**`, and package/release configuration;
- production and test Rust under `src/**` and `tests/**/*.rs`;
- Python executable contracts under `tests/**/*.py`;
- public `README.md`, `docs/**`, `skills/**`, `spec/**`, examples, and schemas;
- current planning and active tickets under `sdlc/planning/clinical-trial-capabilities.md` and `sdlc/tickets/`.

Add a case-insensitive static source/package ratchet that fails on `biodata`, the BioData Git URL/revision, Cargo source/package entries, checkout/path/patch dependencies, or deferred-package exceptions in those active surfaces. Its allowlist is structural and narrow: completed `sdlc/records/**`, `sdlc/tickets/archive/**`, the dated historical planning note, Git history, and the existing fixture provenance receipt may retain truthful BioData prose. The ratchet must have self-tests proving a forbidden manifest dependency, Rust import, test handoff, public-doc claim, active-ticket dependency, Git URL, mixed-case spelling, and lockfile source are rejected, while an allowed historical record/provenance mention passes. It must run in `make lint` through the existing quality-ratchet lane and in source-package tests.

Replace `tests/test_source_package_boundary.py`'s exact-revision/compile-deferral assertions with positive no-dependency and extracted-package verification. `cargo metadata --locked --format-version 1` and `cargo tree --locked` must contain no BioData package, URL, or source edge. `tools/run-offline -- cargo package --locked` must build and verify the extracted package without network access or a sibling repository. Inspect the produced package/archive as well as the worktree so a renamed, generated, or packaged dependency cannot bypass the source scan.

Update the two current user-guide ownership statements to describe BioMCP's local trial contract. Update the current clinical-trial capability plan. Remove the stale active 1180 and 1181 ticket copies because their completion records are already authoritative. Move 1182 to the archive with a concise superseded-by-1183 note; its still-relevant NCI public-boundary proof belongs in this ticket's acceptance. At completion move this ticket's durable result to `sdlc/records/1183-remove-biodata-from-biomcp-0-9.md`, leaving no BioData-named active ticket to defeat the active-surface ratchet. Do not edit or delete completed records, archived historical facts, the dated planning note, capture receipts, provider fixture bytes, or fixture authorship/license provenance.

## Done, observably

- The completed dependency graph has no BioData node or edge, and a clean build needs no BioData repository, package, network fetch, secret, or credential.
- Exact CTGov construction tests prove the current path, deterministic field query, default eligibility, every single section, `all`, documents interaction, and omission of unrelated fields. Exact NCI construction tests prove path, ordered repeated includes, optional eligibility, one credential header, and credential-free plan data.
- Local parser tests retain every current positive and negative case for malformed/duplicate/unsupported JSON, identity mismatch, NCI row cardinality, old lenient shapes, strict required fields, arm ambiguity and relationships, section states, eligibility ordering/classification, structured references, and sanitized failures.
- Resource tests accept every limit exactly at its ceiling and reject the first byte, depth, character, element, member, and event over the ceiling with `json_resource_limit` and narrow recovery.
- Local value tests prove every relationship-error variant, endpoint/duplicate validation, portable identity bounds, eligibility/reference exact JSON, null/empty distinctions, Unicode/order preservation, duplicate nested members, invalid no-limit rules, and data-free errors.
- `tests/test_ctgov_trial_search_detail_reuse.py`, the NCI public-boundary proof superseding 1182, typed/raw MCP trial tests, `tests/json_error_contract.rs`, Markdown tests, schemas, examples, and mustmatch trial specs prove unchanged successful CLI/MCP output and no partial output on failure. Red tests fail when description, `display_order`, classification, malformed-as-absence, assignment, reference fallback, request field, or resource-limit behavior is bypassed.
- The zero-coupling ratchet covers the worktree and produced source package, has the named bypass tests, and allows only the historical/provenance paths above.
- On Linux, `make lint`, `make test`, `make spec`, `make full-feature-check`, and the complete `make release-gate` pass under the repository-pinned toolchain and Node 22 requirements.
- The `windows-contracts` lane passes `cargo check --locked` and `cargo test --locked --test managed_state_permissions` on `windows-2022`; the release workflow also builds and inspects the `x86_64-pc-windows-msvc` candidate without private dependency access.
- A clean/offline `cargo metadata`, `cargo tree`, package verification, and extracted-package compile all pass, and their output contains no forbidden dependency evidence.

## Boundaries

This ticket does not change trial search, provider URLs, public success JSON or Markdown, schemas, section names, source attribution, fixtures, capture receipts, release version, unrelated dependencies, or AlphaGenome behavior. It does not publish a release, BioData crate, package, branch, or repository; contact external systems; add CI credentials; or rewrite history. It does not import BioData source under another name or broaden BioMCP into a general biomedical data library.

## Dependencies

None. The required provider fixtures, product contracts, historical implementation evidence, and current behavior tests are already in BioMCP. BioData access is neither a prerequisite nor permitted for implementation or gates.

## Review

- Design review: pending
- Code review: pending
